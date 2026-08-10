//! Oturum omru olcumu ve otomatik yeniden giris.
//!
//! Aternos cerezleri tam 30 gun yasiyor (olculdu: `Max-Age=2592000`). Hesap
//! bilgileri saklandiysa daemon suresi dolunca KENDI giris yapar ve kullanici
//! hicbir sey fark etmez; saklanmadiysa panel cerez yenilemeye cagirir.
//!
//! Ayrica her oturumun ne kadar dayandigini yazar: Aternos bir sayi ilan
//! etmedigi icin her kurulum kendi verisini toplar.

use crate::config;
use crate::web::{self, AppCtx};
use aterkeep_core::SharedState;
use std::sync::Arc;

/// Gorevin ihtiyac duydugu her sey. Ayri bir struct: parametre listesi
/// uzadikca cagri yerinde hangi degerin hangi alana gittigi belirsizlesiyordu.
pub struct Deps {
    pub state: SharedState,
    pub ctx: Arc<AppCtx>,
    pub tx: aterkeep_core::LogTx,
    /// Yeni oturumu ayni anahtarla yeniden sifrelemek icin.
    pub session_key: [u8; 32],
    /// Yeniden baslatirken cocuk surece ATERKEEP_KEY olarak aktarilir.
    pub panel_password: String,
}

pub fn spawn(deps: Deps) {
    let Deps { state, ctx: watch_ctx, tx, session_key, panel_password } = deps;
    let session_key = Some(session_key);
    let panel_password = Some(panel_password);

        tokio::spawn(async move {
            const TICK: u64 = 20;
            // Omur olcumu yalnizca gecis aninda yazilir; yeniden giris ise
            // BASARANA KADAR denenir. Onceden ikisi ayni bayraga baglilydi:
            // tek bir gecici ag hatasi otomatik yenilemeyi kalici olarak
            // kapatiyor, kullanici 30 gun sonra paneli olu buluyordu.
            let mut lifetime_recorded = false;
            let mut wait_secs: u64 = 0;
            let mut backoff: u64 = 60;
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(TICK)).await;
                let expired = { state.lock().await.session_expired };
                if !expired {
                    lifetime_recorded = false;
                    wait_secs = 0;
                    backoff = 60;
                    continue;
                }
                // Yeniden deneme araligi dolmadiysa bekle.
                if wait_secs > 0 {
                    wait_secs = wait_secs.saturating_sub(TICK);
                    continue;
                }
                if !lifetime_recorded {
                    lifetime_recorded = true;
                    {
                        let mut cfg = watch_ctx.cfg.lock().await;
                        if let Some(started) = cfg.session_started {
                            let lifetime = web::now_unix().saturating_sub(started);
                            cfg.last_session_lifetime = Some(lifetime);
                            let _ = cfg.save();
                            aterkeep_core::log(
                                &tx,
                                "warn",
                                format!(
                                    "oturum {} gun {} saat dayandi",
                                    lifetime / 86_400,
                                    (lifetime % 86_400) / 3_600
                                ),
                            );
                        }
                    }
                }
                {
                    // Hesap bilgileri saklanmissa oturumu KENDIMIZ yenileriz.
                    // Kullanicinin 30 gunde bir DevTools acip cerez kopyalamasi
                    // gerekmez — urunun asil vaadi bu.
                    let creds = {
                        let s = watch_ctx.client.session.clone();
                        s.username.zip(s.password)
                    };
                    match creds {
                        Some((u, p)) => {
                            aterkeep_core::log(
                                &tx,
                                "sys",
                                "cerezler doldu — hesapla yeniden giris yapiliyor".into(),
                            );
                            let sid = watch_ctx.client.session.server_id.clone();
                            match aterkeep_core::login(&u, &p, Some(sid)).await {
                                Ok(mut fresh) => {
                                    fresh.username = Some(u);
                                    fresh.password = Some(p);
                                    match session_key {
                                        Some(k) => {
                                            if let Err(e) = fresh
                                                .save_encrypted(&config::session_path(), &k)
                                            {
                                                aterkeep_core::log(
                                                    &tx,
                                                    "err",
                                                    format!("yeni oturum yazilamadi: {e}"),
                                                );
                                            } else {
                                                let mut cfg = watch_ctx.cfg.lock().await;
                                                cfg.session_started = Some(web::now_unix());
                                                let _ = cfg.save();
                                                aterkeep_core::log(
                                                    &tx,
                                                    "ok",
                                                    "yeni oturum alindi — daemon yeniden basliyor"
                                                        .into(),
                                                );
                                                // Client'in oturumu process omru
                                                // boyunca sabit; taze cerezle
                                                // devam etmenin en basit ve en
                                                // az riskli yolu yeniden baslamak.
                                                tokio::time::sleep(
                                                    std::time::Duration::from_millis(500),
                                                )
                                                .await;
                                                web::self_restart(panel_password.as_deref());
                                            }
                                        }
                                        None => aterkeep_core::log(
                                            &tx,
                                            "err",
                                            "oturum anahtari yok — otomatik yenileme yapilamadi"
                                                .into(),
                                        ),
                                    }
                                }
                                Err(e) => {
                                    // Ustel geri cekilme, sonra TEKRAR DENE.
                                    // Vazgecmek, kullaniciyi 30 gun sonra olu
                                    // bir panelle bas basa birakmak demek.
                                    wait_secs = backoff;
                                    backoff = (backoff * 2).min(1800);
                                    aterkeep_core::log(
                                        &tx,
                                        "err",
                                        format!(
                                            "otomatik giris basarisiz: {e} — {wait_secs}sn sonra tekrar"
                                        ),
                                    );
                                }
                            }
                        }
                        None => {
                            // Hesap yok: her 20 saniyede bir ayni uyariyi
                            // basmayalim, saatte bir yeter.
                            wait_secs = 3600;
                            aterkeep_core::log(
                                &tx,
                                "warn",
                                "hesap bilgisi saklanmamis — cerezleri panelden yenilemen gerekiyor"
                                    .into(),
                            );
                        }
                    }
                }
            }
        });
}
