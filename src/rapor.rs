//! Haftalik rapor: gun bazli ozet, JSON ve Markdown cikti.
//!
//! Rapor, oturum gunlugunden ve gorev dosyasindan **yeniden uretilir**; ayni
//! girdiyle ayni sayilar uretilir. Gorev "tamamlandi/ertelendi" sayimi, gorevin
//! `guncelleme` zaman damgasinin rapor araligina dustugu gunlardan hesaplanir
//! (gorev gecmisi tutulmadigi icin bu bir yaklasimdir; README'de belgelenir).

use serde::{Deserialize, Serialize};

use crate::gorev::{Durum, GorevDeposu};
use crate::hata::Sonuc;
use crate::oturum::OturumGunlugu;
use crate::zaman::{hafta_gunleri, hafta_yaz, sure_yaz, tarih_yaz};

/// Tek bir gunun ozeti.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GunOzeti {
    /// `YYYY-MM-DD`.
    pub tarih: String,
    /// Gunun adi (Turkce).
    pub gun_adi: String,
    /// Gundeki oturum sayisi.
    pub oturum_sayisi: u32,
    /// Gundeki toplam **odak** suresi (dakika).
    pub odak_dakika: i32,
    /// Gundeki toplam kesinti suresi (dakika).
    pub kesinti_dakika: i32,
    /// Gunde tamamlanan gorev sayisi.
    pub tamamlanan: u32,
    /// Gunde ertelenen gorev sayisi.
    pub ertelenen: u32,
    /// Gunde tamamlanan gorevlerin toplam tahmini suresi (dakika).
    pub tahmini_dakika: i32,
}

impl GunOzeti {
    fn bos(tarih: String, gun_adi: String) -> Self {
        Self {
            tarih,
            gun_adi,
            oturum_sayisi: 0,
            odak_dakika: 0,
            kesinti_dakika: 0,
            tamamlanan: 0,
            ertelenen: 0,
            tahmini_dakika: 0,
        }
    }
}

/// Yedi gunluk haftalik rapor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HaftaRaporu {
    /// `YYYY-Www`.
    pub hafta: String,
    /// Raporun ilk gununun tarihi.
    pub ilk_gun: String,
    /// Raporun son gununun tarihi.
    pub son_gun: String,
    /// Gun bazli ozetler (yedi tane).
    pub gunler: Vec<GunOzeti>,
    /// Toplam oturum sayisi.
    pub toplam_oturum: u32,
    /// Toplam odak suresi (dakika).
    pub toplam_odak_dakika: i32,
    /// Toplam kesinti suresi (dakika).
    pub toplam_kesinti_dakika: i32,
    /// Toplam tamamlanan gorev sayisi.
    pub toplam_tamamlanan: u32,
    /// Toplam ertelenen gorev sayisi.
    pub toplam_ertelenen: u32,
    /// Tamamlanan gorevlerin toplam tahmini suresi (dakika).
    pub toplam_tahmini_dakika: i32,
    /// Kesinti kaynaklarinin toplam suresi, alfabetik sirali.
    pub kaynaklar: Vec<(String, i32)>,
}

impl HaftaRaporu {
    /// Tahmin ile gerceklesen arasindaki oran (yuzde).
    ///
    /// Tahmin girilmemisse 0 doner; bu durumda oran **olculmemis** demektir.
    pub fn gerceklesme_yuzdesi(&self) -> i32 {
        if self.toplam_tahmini_dakika <= 0 {
            return 0;
        }
        self.toplam_odak_dakika * 100 / self.toplam_tahmini_dakika
    }

    /// Tek satirlik terminal ozeti.
    pub fn ozet_satiri(&self) -> String {
        format!(
            "{} ({} .. {}): {} oturum, {} odak, {} kesinti, {} tamamlanan, {} ertelenen",
            self.hafta,
            self.ilk_gun,
            self.son_gun,
            self.toplam_oturum,
            sure_yaz(self.toplam_odak_dakika),
            sure_yaz(self.toplam_kesinti_dakika),
            self.toplam_tamamlanan,
            self.toplam_ertelenen
        )
    }

    /// JSON ciktisi (guzel basili).
    pub fn json(&self) -> Sonuc<String> {
        serde_json::to_string_pretty(self).map_err(|hata| crate::hata::Hata::BozukJson {
            ayrinti: hata.to_string(),
        })
    }

    /// Markdown cikti.
    ///
    /// Tahmin ve gerceklesen yan yana gosterilir; sifira bolme yapilmaz.
    pub fn markdown(&self) -> String {
        let mut satirlar: Vec<String> = Vec::new();
        satirlar.push(format!("# FocusCompass haftalik rapor - {}", self.hafta));
        satirlar.push(String::new());
        satirlar.push(format!("Donem: {} .. {}", self.ilk_gun, self.son_gun));
        satirlar.push(format!("- Toplam oturum: {}", self.toplam_oturum));
        satirlar.push(format!(
            "- Toplam odak suresi: {}",
            sure_yaz(self.toplam_odak_dakika)
        ));
        satirlar.push(format!(
            "- Toplam kesinti suresi: {}",
            sure_yaz(self.toplam_kesinti_dakika)
        ));
        satirlar.push(format!("- Tamamlanan gorev: {}", self.toplam_tamamlanan));
        satirlar.push(format!("- Ertelenen gorev: {}", self.toplam_ertelenen));
        satirlar.push(format!(
            "- Tahmin (tamamlananlar): {}",
            sure_yaz(self.toplam_tahmini_dakika)
        ));
        satirlar.push(format!("- Gerceklesme: %{}", self.gerceklesme_yuzdesi()));
        satirlar.push(String::new());
        satirlar.push("## Gunluk ozet".to_string());
        satirlar.push(String::new());
        satirlar.push(
            "| Tarih | Gun | Oturum | Odak | Kesinti | Tamamlanan | Ertelenen | Tahmin |"
                .to_string(),
        );
        satirlar.push("|---|---|---|---|---|---|---|---|".to_string());
        for gun in &self.gunler {
            satirlar.push(format!(
                "| {} | {} | {} | {} | {} | {} | {} | {} |",
                gun.tarih,
                gun.gun_adi,
                gun.oturum_sayisi,
                gun.odak_dakika,
                gun.kesinti_dakika,
                gun.tamamlanan,
                gun.ertelenen,
                gun.tahmini_dakika
            ));
        }
        satirlar.push(String::new());
        satirlar.push("## Kesinti kaynaklari".to_string());
        satirlar.push(String::new());
        if self.kaynaklar.is_empty() {
            satirlar.push("Kayitli kesinti yok.".to_string());
        } else {
            satirlar.push("| Kaynak | Sure |".to_string());
            satirlar.push("|---|---|".to_string());
            for (kaynak, dakika) in &self.kaynaklar {
                satirlar.push(format!("| {kaynak} | {dakika} dk |"));
            }
        }
        satirlar.push(String::new());
        satirlar.push(
            "> Bu rapor yerel dosyalardan yeniden uretilir; ayni girdiyle ayni sayilari verir."
                .to_string(),
        );
        satirlar.join("\n")
    }

    /// Dosya adlari: `hafta-2026-W40.json` ve `hafta-2026-W40.md`.
    pub fn dosya_adi(&self, uzanti: &str) -> String {
        format!("hafta-{}.{uzanti}", self.hafta)
    }
}

/// Haftalik raporu uretir.
///
/// `hafta_pazartesi` bir ISO haftasinin Pazartesi gun sayisidir; `gun_saniye`
/// bir gunun uzunlugudur (sabit offerde 86.400).
pub fn rapor_olustur(
    hafta_pazartesi: i64,
    gun_saniye: i64,
    gunluk: &OturumGunlugu,
    depo: &GorevDeposu,
) -> Sonuc<HaftaRaporu> {
    let mut gunler: Vec<GunOzeti> = Vec::new();
    for gun_no in hafta_gunleri(hafta_pazartesi) {
        let bas = gun_no * gun_saniye;
        let son = bas + gun_saniye;
        let oturumlar = gunluk.gun_ici(bas, gun_saniye);
        let mut ozet = GunOzeti::bos(tarih_yaz(gun_no), crate::zaman::gun_adi(gun_no).to_string());
        ozet.oturum_sayisi = oturumlar.len() as u32;
        ozet.odak_dakika = gunluk.toplam_odak_dakika(bas, son)?;
        ozet.kesinti_dakika = gunluk.toplam_kesinti_dakika(bas, son);
        for gorev in depo.tum() {
            if gorev.guncelleme < bas || gorev.guncelleme >= son {
                continue;
            }
            match gorev.durum {
                Durum::Tamamlandi => {
                    ozet.tamamlanan += 1;
                    ozet.tahmini_dakika += gorev.tahmini_dakika;
                }
                Durum::Ertelendi => ozet.ertelenen += 1,
                _ => {}
            }
        }
        gunler.push(ozet);
    }

    let ilk = hafta_pazartesi;
    let son = hafta_pazartesi + 6;
    let toplam_oturum = gunler.iter().map(|g| g.oturum_sayisi).sum();
    let kaynaklar = gunluk.kaynak_dagilimi(ilk * gun_saniye, (son + 1) * gun_saniye);

    Ok(HaftaRaporu {
        hafta: hafta_yaz(ilk),
        ilk_gun: tarih_yaz(ilk),
        son_gun: tarih_yaz(son),
        toplam_oturum,
        toplam_odak_dakika: gunler.iter().map(|g| g.odak_dakika).sum(),
        toplam_kesinti_dakika: gunler.iter().map(|g| g.kesinti_dakika).sum(),
        toplam_tamamlanan: gunler.iter().map(|g| g.tamamlanan).sum(),
        toplam_ertelenen: gunler.iter().map(|g| g.ertelenen).sum(),
        toplam_tahmini_dakika: gunler.iter().map(|g| g.tahmini_dakika).sum(),
        gunler,
        kaynaklar,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::gorev::{GorevGirdi, Oncelik};
    use crate::oturum::OturumTuru;
    use crate::zaman::{gun_no_uret, tarih_yaz};

    const GUN: i64 = 86_400;

    fn depo() -> GorevDeposu {
        GorevDeposu::yeni()
    }

    /// Haftanin Pazartesi gun sayisini dondurur.
    fn pazartesi() -> i64 {
        crate::zaman::hafta_ayir("2026-W40").unwrap()
    }

    fn oturum_ekle(
        gunluk: &mut OturumGunlugu,
        gorev: Option<u32>,
        gun_no: i64,
        saat_dk: i64,
        planli: i32,
    ) {
        let bas = gun_no * GUN + saat_dk * 60;
        let sira = gunluk.baslat(gorev, OturumTuru::Odak, planli, bas).unwrap();
        gunluk
            .getir_mut(sira)
            .unwrap()
            .bitir(bas + i64::from(planli) * 60)
            .unwrap();
    }

    #[test]
    fn hafta_yedi_gun_doner_ve_aralik_dogru() {
        let gunluk = OturumGunlugu::yeni();
        let d = depo();
        let rapor = rapor_olustur(pazartesi(), GUN, &gunluk, &d).unwrap();
        assert_eq!(rapor.hafta, "2026-W40");
        assert_eq!(rapor.gunler.len(), 7);
        assert_eq!(rapor.ilk_gun, "2026-09-28");
        assert_eq!(rapor.son_gun, "2026-10-04");
    }

    #[test]
    fn haftalik_rapor_toplamlari_dogru() {
        let pzt = pazartesi();
        let mut gunluk = OturumGunlugu::yeni();
        oturum_ekle(&mut gunluk, Some(1), pzt, 0, 25);
        oturum_ekle(&mut gunluk, Some(1), pzt, 6 * 60, 25);
        oturum_ekle(&mut gunluk, Some(2), pzt + 2, 0, 50);
        let mut d = depo();
        d.ekle(
            GorevGirdi {
                baslik: "A".to_string(),
                tahmini_dakika: 30,
                oncelik: Oncelik::Yuksek,
                etiketler: Vec::new(),
                bagimliliklar: Vec::new(),
            },
            pzt * GUN + 1_000,
        )
        .unwrap();
        d.ekle(
            GorevGirdi {
                baslik: "B".to_string(),
                tahmini_dakika: 20,
                oncelik: Oncelik::Normal,
                etiketler: Vec::new(),
                bagimliliklar: Vec::new(),
            },
            (pzt + 1) * GUN,
        )
        .unwrap();
        d.durum_degistir(1, Durum::Tamamlandi, pzt * GUN + 2_000)
            .unwrap();
        d.durum_degistir(2, Durum::Ertelendi, (pzt + 1) * GUN)
            .unwrap();

        let rapor = rapor_olustur(pzt, GUN, &gunluk, &d).unwrap();
        assert_eq!(rapor.toplam_oturum, 3);
        assert_eq!(rapor.toplam_odak_dakika, 100);
        assert_eq!(rapor.toplam_tamamlanan, 1);
        assert_eq!(rapor.toplam_ertelenen, 1);
        assert_eq!(rapor.toplam_tahmini_dakika, 30);
        assert_eq!(rapor.gunler[0].odak_dakika, 50);
        assert_eq!(rapor.gunler[2].odak_dakika, 50);
        assert_eq!(rapor.gunler[3].odak_dakika, 0);
    }

    #[test]
    fn kesinti_toplamlari_ve_kaynak_dagilimi() {
        let pzt = pazartesi();
        let mut gunluk = OturumGunlugu::yeni();
        let bas = pzt * GUN;
        let sira = gunluk.baslat(Some(1), OturumTuru::Odak, 25, bas).unwrap();
        {
            let oturum = gunluk.getir_mut(sira).unwrap();
            oturum.kesinti_ekle("toplanti", 5, 25).unwrap();
            oturum.kesinti_ekle("mesaj", 2, 25).unwrap();
            oturum.bitir(bas + 25 * 60).unwrap();
        }
        let rapor = rapor_olustur(pzt, GUN, &gunluk, &depo()).unwrap();
        assert_eq!(rapor.toplam_kesinti_dakika, 7);
        assert_eq!(rapor.toplam_odak_dakika, 18);
        assert_eq!(
            rapor.kaynaklar,
            vec![("mesaj".to_string(), 2), ("toplanti".to_string(), 5)]
        );
    }

    #[test]
    fn gerceklesme_yuzdesi_tahminle_hesaplanir() {
        let pzt = pazartesi();
        let mut gunluk = OturumGunlugu::yeni();
        oturum_ekle(&mut gunluk, Some(1), pzt, 0, 30);
        let mut d = depo();
        d.ekle(
            GorevGirdi {
                baslik: "A".to_string(),
                tahmini_dakika: 60,
                oncelik: Oncelik::Yuksek,
                etiketler: Vec::new(),
                bagimliliklar: Vec::new(),
            },
            pzt * GUN,
        )
        .unwrap();
        d.durum_degistir(1, Durum::Tamamlandi, pzt * GUN + 1)
            .unwrap();
        let rapor = rapor_olustur(pzt, GUN, &gunluk, &d).unwrap();
        assert_eq!(rapor.gerceklesme_yuzdesi(), 50);
    }

    #[test]
    fn tahmin_yokken_yuzde_sifirdir() {
        let rapor = rapor_olustur(pazartesi(), GUN, &OturumGunlugu::yeni(), &depo()).unwrap();
        assert_eq!(rapor.gerceklesme_yuzdesi(), 0);
        assert!(rapor.ozet_satiri().contains("0 dk odak"));
    }

    #[test]
    fn markdown_tahmin_ve_gercekleseni_yan_yana_gosterir() {
        let pzt = pazartesi();
        let mut gunluk = OturumGunlugu::yeni();
        oturum_ekle(&mut gunluk, Some(1), pzt, 0, 30);
        let mut d = depo();
        d.ekle(
            GorevGirdi {
                baslik: "A".to_string(),
                tahmini_dakika: 60,
                oncelik: Oncelik::Yuksek,
                etiketler: Vec::new(),
                bagimliliklar: Vec::new(),
            },
            pzt * GUN,
        )
        .unwrap();
        d.durum_degistir(1, Durum::Tamamlandi, pzt * GUN + 1)
            .unwrap();
        let rapor = rapor_olustur(pzt, GUN, &gunluk, &d).unwrap();
        let md = rapor.markdown();
        assert!(
            md.contains("# FocusCompass haftalik rapor - 2026-W40"),
            "{md}"
        );
        assert!(md.contains("- Tahmin (tamamlananlar): 1 sa"), "{md}");
        assert!(md.contains("- Gerceklesme: %50"), "{md}");
        assert!(md.contains("| Tarih | Gun | Oturum |"), "{md}");
        assert!(md.contains("## Kesinti kaynaklari"), "{md}");
        assert_eq!(rapor.dosya_adi("md"), "hafta-2026-W40.md");
        assert_eq!(rapor.dosya_adi("json"), "hafta-2026-W40.json");
    }

    #[test]
    fn json_rapor_yapisal_gecerli_ve_geri_yuklenebilir() {
        let mut gunluk = OturumGunlugu::yeni();
        oturum_ekle(&mut gunluk, Some(1), pazartesi(), 0, 25);
        let rapor = rapor_olustur(pazartesi(), GUN, &gunluk, &depo()).unwrap();
        let metin = rapor.json().unwrap();
        let geri: HaftaRaporu = serde_json::from_str(&metin).unwrap();
        assert_eq!(geri, rapor);
        assert!(metin.contains("\"toplam_odak_dakika\": 25"));
    }

    #[test]
    fn rapor_ayni_girdiyle_ayni_sayilari_uretir() {
        let pzt = pazartesi();
        let mut gunluk = OturumGunlugu::yeni();
        oturum_ekle(&mut gunluk, Some(1), pzt, 0, 25);
        oturum_ekle(&mut gunluk, Some(1), pzt + 1, 0, 25);
        let d = depo();
        let a = rapor_olustur(pzt, GUN, &gunluk, &d).unwrap();
        let b = rapor_olustur(pzt, GUN, &gunluk, &d).unwrap();
        assert_eq!(a, b, "rapor tekrarlanabilir olmali");
    }

    #[test]
    fn hafta_disi_oturumlar_raporu_etkilemez() {
        let pzt = pazartesi();
        let mut gunluk = OturumGunlugu::yeni();
        oturum_ekle(&mut gunluk, Some(1), pzt - 3, 0, 25);
        oturum_ekle(&mut gunluk, Some(1), pzt, 0, 25);
        oturum_ekle(&mut gunluk, Some(1), pzt + 7, 0, 25);
        let rapor = rapor_olustur(pzt, GUN, &gunluk, &depo()).unwrap();
        assert_eq!(rapor.toplam_oturum, 1);
        assert_eq!(rapor.toplam_odak_dakika, 25);
    }

    #[test]
    fn gun_adi_ve_tarih_uyumu() {
        let rapor = rapor_olustur(pazartesi(), GUN, &OturumGunlugu::yeni(), &depo()).unwrap();
        assert_eq!(rapor.gunler[0].gun_adi, "Pazartesi");
        assert_eq!(rapor.gunler[6].gun_adi, "Pazar");
        assert_eq!(
            rapor.gunler[0].tarih,
            tarih_yaz(gun_no_uret(2026, 9, 28).unwrap())
        );
    }
}
