//! Uygulama genelinde kullanilan tek hata turu ve onun `Display` uygulamasi.
//!
//! `thiserror` bagimliligi yasak oldugu icin `Display` ve `std::error::Error`
//! uygulamalari elle yazildi. Kullanici girdisinden kaynaklanan her durum
//! `Result` ile doner; `panic!` uretim kodunda kullanilmaz.

use std::fmt;
use std::path::PathBuf;

use crate::gorev::Durum;

/// FocusCompass'in urettigi tum hatalar.
#[derive(Debug)]
#[non_exhaustive]
pub enum Hata {
    /// Veri dosyasindaki sema surumu bu surumle uyusmuyor.
    SurumUyusmazligi {
        /// Dosyada bulunan surum.
        bulunan: u32,
        /// Bu surumun anladigi surum.
        beklenen: u32,
    },
    /// JSON metni ayristirilamadi.
    BozukJson {
        /// Ayristirici hatasi.
        ayrinti: String,
    },
    /// JSONL gunlugunde tek bir satir ayristirilamadi.
    BozukSatir {
        /// Hatali satirin bulundugu dosya.
        yol: PathBuf,
        /// Hatali satirin 1 tabanli numarasi.
        satir_no: usize,
        /// Ayristirici hatasi.
        ayrinti: String,
    },
    /// Dosya sistemi islemi basarisiz oldu.
    Dosya {
        /// Islem yapilan yol.
        yol: PathBuf,
        /// Isletim sistemi hata metni.
        ayrinti: String,
    },
    /// Gorev basligi bos veya yalnizca bosluk iceriyor.
    BosBaslik,
    /// Durum adi `bekliyor|calisiyor|tamamlandi|ertelendi` disinda.
    GecersizDurumAdi {
        /// Girilen metin.
        metin: String,
    },
    /// Oncelik adi `dusuk|normal|yuksek` disinda.
    GecersizOncelik {
        /// Girilen metin.
        metin: String,
    },
    /// Ayni gorev kimligi iki kez tanimli.
    TekrarliKimlik {
        /// Tekrar eden kimlik.
        id: u32,
    },
    /// Tahmini sure negatif.
    NegatifSure {
        /// Girilen deger.
        deger: i32,
    },
    /// Sure sifir; oturum veya gorev icin anlamsiz.
    SifirSure,
    /// Etiket bos ya da izin verilmeyen karakter iceriyor.
    GecersizEtiket {
        /// reddedilen etiket.
        etiket: String,
    },
    /// Durum makinesi bu gecise izin vermiyor.
    GecersizGecis {
        /// Mevcut durum.
        kaynak: Durum,
        /// Istenen durum.
        hedef: Durum,
    },
    /// Verilen kimlikte gorev yok.
    BilinmeyenGorev {
        /// Aranan kimlik.
        id: u32,
    },
    /// Ayni bagimlilik zaten kayitli.
    TekrarliBagimlilik {
        /// Bagimlilik eklenen gorev.
        id: u32,
        /// Bagimlilik olan gorev.
        bagimlilik: u32,
    },
    /// Gorev kendine bagimli yapildi.
    KendineBagimlilik {
        /// Kendine baglanan gorev.
        id: u32,
    },
    /// Bagimlilik grafiginde dongu olustu.
    BagimlilikDongusu {
        /// Donguyu olusturan kimlikler, gorevden goreve sirali.
        yol: Vec<u32>,
    },
    /// Bagimlilik olarak verilen gorev depoda bulunamadi.
    BilinmeyenBagimlilik {
        /// Bagimlilik eklenen gorev.
        id: u32,
        /// Var olmayan bagimlilik.
        bagimlilik: u32,
    },
    /// Gorevin tamamlanmamis bagimliliklari var.
    Baslatilamaz {
        /// Baslatilamayan gorev.
        id: u32,
        /// Tamamlanmamis bagimliliklar.
        eksik: Vec<u32>,
    },
    /// Baslatilamayan gorevin engellenme gerekcesi.
    BaslatilamazSebep {
        /// Baslatilamayan gorev.
        id: u32,
        /// Engelleyen gorev.
        engelleyen: u32,
        /// Engelleyen gorevin durumu.
        durum: Durum,
    },
    /// Baslatmak icin acik bir pomodoro oturumu zaten var.
    OturumCakisiyor {
        /// Acik kalan oturumun sira numarasi.
        acik_sira: u32,
    },
    /// Kesinti toplami gecen oturum suresini asiyor.
    KesintiAsim {
        /// Kesintilerin toplam dakikası.
        toplam: i32,
        /// Oturumdan gecen toplam dakika.
        gecen: i32,
    },
    /// Kapanmis oturum yeniden kapatilamaz.
    OturumKapatilmis {
        /// Oturumun sira numarasi.
        sira: u32,
    },
    /// Bitis zamani baslangictan once.
    BitisBaslangictanOnce {
        /// Oturumun sira numarasi.
        sira: u32,
    },
    /// Kapatilacak acik oturum bulunamadi.
    AcikOturumYok,
    /// Tarih metni `YYYY-MM-DD` biciminde degil.
    GecersizTarih {
        /// Girilen metin.
        girdi: String,
    },
    /// Hafta metni `YYYY-Www` biciminde degil.
    GecersizHafta {
        /// Girilen metin.
        girdi: String,
    },
    /// Saat dilimi ofseti -14:00 .. +14:00 araliginda degil.
    GecersizSaatDilimi {
        /// Girilen ofset (dakika).
        ofset_dakika: i32,
    },
    /// Kapasite veya sabit sure negatif.
    GecersizKapasite {
        /// Girilen deger.
        deger: i32,
    },
    /// Cikti bicimi taninmiyor.
    GecersizFormat {
        /// Girilen metin.
        girdi: String,
    },
}

impl fmt::Display for Hata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SurumUyusmazligi { bulunan, beklenen } => write!(
                f,
                "veri dosyasi surumu uyusmuyor: dosyada {bulunan}, bu surum {beklenen} bekliyor"
            ),
            Self::BozukJson { ayrinti } => write!(f, "bozuk JSON: {ayrinti}"),
            Self::BozukSatir {
                yol,
                satir_no,
                ayrinti,
            } => write!(
                f,
                "bozuk JSONL satiri {}: satir {satir_no} ({ayrinti})",
                yol.display()
            ),
            Self::Dosya { yol, ayrinti } => {
                write!(f, "dosya islemi basarisiz ({}): {ayrinti}", yol.display())
            }
            Self::BosBaslik => write!(f, "gorev basligi bos olamaz"),
            Self::GecersizDurumAdi { metin } => write!(
                f,
                "gecersiz durum: '{metin}' (bekliyor | calisiyor | tamamlandi | ertelendi)"
            ),
            Self::GecersizOncelik { metin } => {
                write!(f, "gecersiz oncelik: '{metin}' (dusuk | normal | yuksek)")
            }
            Self::TekrarliKimlik { id } => write!(f, "gorev kimligi tekrarli: #{id}"),
            Self::NegatifSure { deger } => write!(f, "sure negatif olamaz: {deger} dk"),
            Self::SifirSure => write!(f, "sure sifir olamaz"),
            Self::GecersizEtiket { etiket } => {
                write!(
                    f,
                    "gecersiz etiket: '{etiket}' (harf, rakam, '-' ve '_' kullanilir)"
                )
            }
            Self::GecersizGecis { kaynak, hedef } => write!(
                f,
                "gecersiz durum gecisi: {} -> {}",
                kaynak.ad(),
                hedef.ad()
            ),
            Self::BilinmeyenGorev { id } => write!(f, "gorev bulunamadi: #{id}"),
            Self::TekrarliBagimlilik { id, bagimlilik } => {
                write!(f, "bagimlilik zaten kayitli: #{id} -> #{bagimlilik}")
            }
            Self::KendineBagimlilik { id } => write!(f, "gorev kendine baglanamaz: #{id}"),
            Self::BagimlilikDongusu { yol } => {
                write!(f, "bagimlilik dongusu: {}", yol_adet(yol))
            }
            Self::BilinmeyenBagimlilik { id, bagimlilik } => {
                write!(f, "bagimlilik gorevi yok: #{id} -> #{bagimlilik}")
            }
            Self::Baslatilamaz { id, eksik } => {
                let liste = kimlik_listesi(eksik);
                write!(
                    f,
                    "gorev baslatilamaz: #{id} (tamamlanmamis bagimlilik: {liste})"
                )
            }
            Self::BaslatilamazSebep {
                id,
                engelleyen,
                durum,
            } => write!(
                f,
                "gorev baslatilamaz: #{id} (bagimlilik #{engelleyen} durumu: {})",
                durum.ad()
            ),
            Self::OturumCakisiyor { acik_sira } => {
                write!(
                    f,
                    "acik pomodoro oturumu var: #{acik_sira}; once onu durdurun"
                )
            }
            Self::KesintiAsim { toplam, gecen } => write!(
                f,
                "kesinti toplami ({toplam} dk) oturumdan gecen {gecen} dk suresini asiyor"
            ),
            Self::OturumKapatilmis { sira } => write!(f, "oturum zaten kapali: #{sira}"),
            Self::BitisBaslangictanOnce { sira } => {
                write!(f, "bitis baslangictan once: #{sira}")
            }
            Self::AcikOturumYok => write!(f, "acik pomodoro oturumu yok"),
            Self::GecersizTarih { girdi } => {
                write!(f, "gecersiz tarih: '{girdi}' (bicim: YYYY-MM-DD)")
            }
            Self::GecersizHafta { girdi } => {
                write!(f, "gecersiz hafta: '{girdi}' (bicim: YYYY-Www)")
            }
            Self::GecersizSaatDilimi { ofset_dakika } => {
                write!(
                    f,
                    "gecersiz saat dilimi: {ofset_dakika} dk (-840..=840 araliginda olmali)"
                )
            }
            Self::GecersizKapasite { deger } => {
                write!(f, "kapasite veya sabit sure negatif olamaz: {deger}")
            }
            Self::GecersizFormat { girdi } => {
                write!(f, "gecersiz format: '{girdi}' (json | md | ikisi)")
            }
        }
    }
}

impl std::error::Error for Hata {}

/// Yorumlama kolayligi icin kimlik listesini `#1, #2` bicimine cevirir.
fn kimlik_listesi(kimlikler: &[u32]) -> String {
    if kimlikler.is_empty() {
        return "-".to_string();
    }
    let parcalar: Vec<String> = kimlikler.iter().map(|k| format!("#{k}")).collect();
    parcalar.join(", ")
}

/// Döngü yolunu okunur hale getirir: `#1 -> #2 -> #1`.
fn yol_adet(yol: &[u32]) -> String {
    if yol.is_empty() {
        return "-".to_string();
    }
    let parcalar: Vec<String> = yol.iter().map(|k| format!("#{k}")).collect();
    parcalar.join(" -> ")
}

/// Yonlendirilen islem sonucunu temsil eden kisaltma.
pub type Sonuc<T> = std::result::Result<T, Hata>;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    #[test]
    fn dongu_hatasi_okunur_yol_yazar() {
        let hata = Hata::BagimlilikDongusu { yol: vec![1, 2, 1] };
        let metin = hata.to_string();
        assert!(metin.contains("#1 -> #2 -> #1"), "metin: {metin}");
    }

    #[test]
    fn gecersiz_gecis_hatasi_iki_durumu_yazar() {
        let hata = Hata::GecersizGecis {
            kaynak: Durum::Tamamlandi,
            hedef: Durum::Ertelendi,
        };
        let metin = hata.to_string();
        assert!(metin.contains("tamamlandi"), "metin: {metin}");
        assert!(metin.contains("ertelendi"), "metin: {metin}");
    }

    #[test]
    fn bos_kimlik_listesi_tire_olsun() {
        assert_eq!(kimlik_listesi(&[]), "-");
        assert_eq!(kimlik_listesi(&[3, 4]), "#3, #4");
    }

    #[test]
    fn dosya_hatasi_yolu_gosterir() {
        let hata = Hata::Dosya {
            yol: PathBuf::from("veri/gorevler.json"),
            ayrinti: "erisim reddedildi".to_string(),
        };
        assert!(hata.to_string().contains("gorevler.json"));
    }
}
