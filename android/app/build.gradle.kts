import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

// Imza: keystore.properties varsa release anahtariyla, yoksa debug anahtariyla
// imzalanir. Amac, gizli anahtar olmadan da KURULABILIR bir APK uretmek —
// imzasiz bir APK Android'e hic yuklenmez, yani "anahtar yoksa imzalamayi atla"
// sessizce ise yaramaz bir cikti verirdi.
val keystoreProps = Properties().apply {
    val f = rootProject.file("keystore.properties")
    if (f.exists()) f.inputStream().use { load(it) }
}
val hasReleaseKey = keystoreProps.getProperty("storeFile") != null

android {
    namespace = "com.karamellis.aterkeep"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.karamellis.aterkeep"
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "1.1.0"

        // Tasidigimiz iki binary (daemon + curl) aarch64 icin derlenmis.
        // Baska bir ABI'yi paketlemek, o cihazda calisamayacak bir APK uretirdi;
        // filtre sayesinde uyumsuz cihaz APK'yi hic kabul etmiyor ve kullanici
        // "kuruldu ama hicbir sey olmuyor" yerine net bir hata goruyor.
        ndk {
            abiFilters.add("arm64-v8a")
        }
    }

    signingConfigs {
        if (hasReleaseKey) {
            create("release") {
                storeFile = rootProject.file(keystoreProps.getProperty("storeFile"))
                storePassword = keystoreProps.getProperty("storePassword")
                keyAlias = keystoreProps.getProperty("keyAlias")
                keyPassword = keystoreProps.getProperty("keyPassword")
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            signingConfig = if (hasReleaseKey) {
                signingConfigs.getByName("release")
            } else {
                signingConfigs.getByName("debug")
            }
        }
    }

    packaging {
        jniLibs {
            // ZORUNLU. Daemon ve curl birer .so degil, CALISTIRILABILIR ELF.
            // Android 10+ uygulama veri dizininde exec'i yasakliyor (W^X);
            // calistirilabilir tek yer, APK'dan diske acilan nativeLibraryDir.
            // useLegacyPackaging=false (AGP 8 varsayilani) .so'lari APK icinde
            // sikistirilmis birakir ve oraya HIC acmaz — o zaman exec edilecek
            // bir dosya olmaz. true => manifest'te extractNativeLibs="true".
            useLegacyPackaging = true
        }
    }

    lint {
        // AGP, `assembleRelease` icine `lintVitalRelease`i baglar ve olumcul
        // sayilan bir bulguda derlemeyi dusurur. Burada lint'in itiraz edecegi
        // seyler BILEREK boyle: minSdk 26 iken manifest'te API 31+ `<property>`
        // etiketi var (eski surumler onu yok sayar) ve pil muafiyeti istiyoruz
        // (BatteryLife). Bir dagitim derlemesinin bu yuzden patlamasi dogru degil.
        checkReleaseBuilds = false
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = "17"
    }
    buildFeatures {
        buildConfig = true
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.15.0")
    implementation("androidx.appcompat:appcompat:1.7.0")
    // Parolayi "hatirla" secenegi icin: Android Keystore destekli sifreli prefs.
    implementation("androidx.security:security-crypto:1.1.0-alpha06")
}
