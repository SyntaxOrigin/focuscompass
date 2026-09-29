//! Gunluk kapasite planlayicisi: "bugun neyi yapamayacagini" soyleyen modul.
//!
//! Kullanici gunluk derin calisma kapasitesini (dakika) ve toplanti/ulasim gibi
//! sabit sureleri girer. Planlayici sabit sureleri duserek kullanilabilir
//! kapasiteyi hesaplar, uygun gorevleri oncelik + tahmini sure + kimlik
//! sirasiyla secer ve **sigmayan her gorevi gerekcesiyle ayri satirda** listeler.
//!
//! Bu modulun agirliklari (oncelik, sure) kullanici girdisine baglidir ve
//! "gercekci plan" iddiasi **olculmemistir**; raporun R2 riski olarak
//! README'de belgelenir.

use serde::{Deserialize, Serialize};

use crate::gorev::{Durum, GorevDeposu, Oncelik};
use crate::hata::{Hata, Sonuc};

/// Bir gun icin kullanici tarafan verilen kapasite girdisi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GunlukKapasite {
    /// Gunun tarihi (`YYYY-MM-DD`); rapor satirlarinda kullanilir.
    pub tarih: String,
    /// Derin calismaya ayrilabilecek toplam sure (dakika).
    pub kapasite_dakika: i32,
    /// Toplanti, ulasim gibi sabit sureler (dakika).
    pub sabit_dakika: i32,
}

impl GunlukKapasite {
    /// Kapasite girdisini olusturur ve negatif degerleri reddeder.
    pub fn yeni(tarih: &str, kapasite_dakika: i32, sabit_dakika: i32) -> Sonuc<Self> {
        if kapasite_dakika < 0 {
            return Err(Hata::GecersizKapasite {
                deger: kapasite_dakika,
            });
        }
        if sabit_dakika < 0 {
            return Err(Hata::GecersizKapasite {
                deger: sabit_dakika,
            });
        }
        Ok(Self {
            tarih: tarih.to_string(),
            kapasite_dakika,
            sabit_dakika,
        })
    }

    /// Sabit sureler dusuldukten sonra kalan kullanilabilir sure (dakika).
    pub fn kullanilabilir_dakika(&self) -> i32 {
        (self.kapasite_dakika - self.sabit_dakika).max(0)
    }
}

/// Bir gorevin plana girmemesinin nedeni.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SigmaSebebi {
    /// Kalan kapasite yetmedi; "bugun sigmaz".
    Kapasite,
    /// Bir bagimlilik henuz tamamlanmadi.
    Bagimlilik,
    /// Gorev zaten calisiyor.
    Calisiyor,
    /// Gorev ertelenmis.
    Ertelendi,
    /// Tahmini sure girilmemis; kapasite hesabina katilamaz.
    TahminYok,
}

impl SigmaSebebi {
    /// Kisa ASCII etiket.
    pub fn etiket(self) -> &'static str {
        match self {
            Self::Kapasite => "kapasite",
            Self::Bagimlilik => "bagimlilik",
            Self::Calisiyor => "calisiyor",
            Self::Ertelendi => "ertelendi",
            Self::TahminYok => "tahmin-yok",
        }
    }
}

/// Planlanan veya plana girmeyen tek bir gorev satiri.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanOgesi {
    /// Gorev kimligi.
    pub gorev_id: u32,
    /// Gorev basligi.
    pub baslik: String,
    /// Tahmini sure (dakika).
    pub tahmini_dakika: i32,
    /// Oncelik.
    pub oncelik: Oncelik,
    /// Sigmayan gorevler icin gerekce.
    pub sebep: Option<SigmaSebebi>,
    /// Gerekcenin insan tarafindan okunur aciklamasi (bos olabilir).
    pub aciklama: String,
}

impl PlanOgesi {
    fn sigacak(gorev_id: u32, baslik: &str, tahmini_dakika: i32, oncelik: Oncelik) -> Self {
        Self {
            gorev_id,
            baslik: baslik.to_string(),
            tahmini_dakika,
            oncelik,
            sebep: None,
            aciklama: String::new(),
        }
    }

    fn sigmayan(
        gorev_id: u32,
        baslik: &str,
        tahmini_dakika: i32,
        oncelik: Oncelik,
        sebep: SigmaSebebi,
        aciklama: String,
    ) -> Self {
        Self {
            gorev_id,
            baslik: baslik.to_string(),
            tahmini_dakika,
            oncelik,
            sebep: Some(sebep),
            aciklama,
        }
    }
}

/// Bir gunun tam plan ciktisi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GunlukPlan {
    /// Gunun tarihi.
    pub tarih: String,
    /// Kullanici tarafan verilen toplam kapasite.
    pub kapasite_dakika: i32,
    /// Dusulen sabit sure.
    pub sabit_dakika: i32,
    /// Kalan kullanilabilir sure.
    pub kullanilabilir_dakika: i32,
    /// Kullanilacak tahmini sure.
    pub kullanilan_dakika: i32,
    /// Kapasiteye sigan gorevler, oncelik sirasiyla.
    pub sigacaklar: Vec<PlanOgesi>,
    /// Sigmayan gorevler, gerekceleriyle birlikte.
    pub sigmayanlar: Vec<PlanOgesi>,
    /// Kullaniciya gosterilen uyarilar.
    pub uyarilar: Vec<String>,
}

impl GunlukPlan {
    /// Kullanilabilir kapasiteden kalan sure.
    pub fn kalan_dakika(&self) -> i32 {
        self.kullanilabilir_dakika - self.kullanilan_dakika
    }

    /// Terminal ciktisi olarak duz metin.
    pub fn metin(&self) -> String {
        let mut satirlar: Vec<String> = Vec::new();
        satirlar.push(format!(
            "{}  kapasite {} dk  sabit {} dk  kullanilabilir {} dk",
            self.tarih, self.kapasite_dakika, self.sabit_dakika, self.kullanilabilir_dakika
        ));
        satirlar.push(String::new());
        satirlar.push(format!(
            "BUGUN SIGANLAR: {} gorev, {} dk",
            self.sigacaklar.len(),
            self.kullanilan_dakika
        ));
        if self.sigacaklar.is_empty() {
            satirlar.push("  (yok)".to_string());
        }
        for oge in &self.sigacaklar {
            satirlar.push(format!(
                "  #{:<4} {:<7} {:>4} dk  {}",
                oge.gorev_id,
                oge.oncelik.ad(),
                oge.tahmini_dakika,
                oge.baslik
            ));
        }
        satirlar.push(String::new());
        satirlar.push(format!(
            "BUGUN SIGMAYANLAR: {} gorev",
            self.sigmayanlar.len()
        ));
        if self.sigmayanlar.is_empty() {
            satirlar.push("  (yok)".to_string());
        }
        for oge in &self.sigmayanlar {
            let sebep = oge.sebep.map_or("-", SigmaSebebi::etiket);
            satirlar.push(format!(
                "  #{:<4} {:>4} dk  {}  [{}]",
                oge.gorev_id, oge.tahmini_dakika, oge.baslik, sebep
            ));
            if !oge.aciklama.is_empty() {
                satirlar.push(format!("        {}", oge.aciklama));
            }
        }
        satirlar.push(String::new());
        satirlar.push(format!(
            "kalan {}",
            crate::zaman::sure_yaz(self.kalan_dakika())
        ));
        if !self.uyarilar.is_empty() {
            satirlar.push(String::new());
            for uyari in &self.uyarilar {
                satirlar.push(format!("uyari: {uyari}"));
            }
        }
        satirlar.join("\n")
    }
}

/// Bir gunluk plan uretir.
///
/// Siralama: oncelik azalan, tahmini sure artan, kimlik artan. Sigmayan gorevler
/// **sessizce dusurulmez**; her biri `sebep` ve `aciklama` ile listelenir.
pub fn planla(depo: &GorevDeposu, kapasite: &GunlukKapasite) -> Sonuc<GunlukPlan> {
    let mut kullanilabilir = kapasite.kullanilabilir_dakika();
    let mut sigacaklar: Vec<PlanOgesi> = Vec::new();
    let mut sigmayanlar: Vec<PlanOgesi> = Vec::new();
    let mut bekleyen_toplam = 0;
    let mut uyarilar: Vec<String> = Vec::new();

    for gorev in depo.sirali() {
        if gorev.durum == Durum::Tamamlandi {
            continue;
        }
        if gorev.durum == Durum::Calisiyor {
            sigmayanlar.push(PlanOgesi::sigmayan(
                gorev.id,
                &gorev.baslik,
                gorev.tahmini_dakika,
                gorev.oncelik,
                SigmaSebebi::Calisiyor,
                "gorev su anda calisiyor".to_string(),
            ));
            continue;
        }
        if gorev.durum == Durum::Ertelendi {
            sigmayanlar.push(PlanOgesi::sigmayan(
                gorev.id,
                &gorev.baslik,
                gorev.tahmini_dakika,
                gorev.oncelik,
                SigmaSebebi::Ertelendi,
                "gorev ertelendi".to_string(),
            ));
            continue;
        }
        let engelleyiciler = depo.tamamlanmamis_bagimliliklar(gorev.id)?;
        if !engelleyiciler.is_empty() {
            let liste: Vec<String> = engelleyiciler
                .iter()
                .map(|(id, durum)| format!("#{id} ({})", durum.ad()))
                .collect();
            sigmayanlar.push(PlanOgesi::sigmayan(
                gorev.id,
                &gorev.baslik,
                gorev.tahmini_dakika,
                gorev.oncelik,
                SigmaSebebi::Bagimlilik,
                format!("tamamlanmamis bagimlilik: {}", liste.join(", ")),
            ));
            continue;
        }
        if gorev.tahmini_dakika == 0 {
            sigmayanlar.push(PlanOgesi::sigmayan(
                gorev.id,
                &gorev.baslik,
                gorev.tahmini_dakika,
                gorev.oncelik,
                SigmaSebebi::TahminYok,
                "tahmini sure girilmemis; kapasite hesabina katilmadi".to_string(),
            ));
            continue;
        }
        bekleyen_toplam += gorev.tahmini_dakika;
        if gorev.tahmini_dakika <= kullanilabilir {
            kullanilabilir -= gorev.tahmini_dakika;
            sigacaklar.push(PlanOgesi::sigacak(
                gorev.id,
                &gorev.baslik,
                gorev.tahmini_dakika,
                gorev.oncelik,
            ));
        } else {
            sigmayanlar.push(PlanOgesi::sigmayan(
                gorev.id,
                &gorev.baslik,
                gorev.tahmini_dakika,
                gorev.oncelik,
                SigmaSebebi::Kapasite,
                format!(
                    "kalan {} yetmiyor (gerekli {})",
                    crate::zaman::sure_yaz(kullanilabilir),
                    crate::zaman::sure_yaz(gorev.tahmini_dakika)
                ),
            ));
        }
    }

    if kapasite.kullanilabilir_dakika() == 0 && bekleyen_toplam > 0 {
        uyarilar.push(format!(
            "kullanilabilir kapasite sifir; {} planlanabilir is hicbir gunune sigmaz",
            crate::zaman::sure_yaz(bekleyen_toplam)
        ));
    } else if bekleyen_toplam > kapasite.kullanilabilir_dakika() {
        uyarilar.push(format!(
            "kapasite asimi: {} planlanabilir is, {} kullanilabilir kapasite",
            crate::zaman::sure_yaz(bekleyen_toplam),
            crate::zaman::sure_yaz(kapasite.kullanilabilir_dakika())
        ));
    }

    Ok(GunlukPlan {
        tarih: kapasite.tarih.clone(),
        kapasite_dakika: kapasite.kapasite_dakika,
        sabit_dakika: kapasite.sabit_dakika,
        kullanilabilir_dakika: kapasite.kullanilabilir_dakika(),
        kullanilan_dakika: kapasite.kullanilabilir_dakika() - kullanilabilir,
        sigacaklar,
        sigmayanlar,
        uyarilar,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::gorev::{GorevDeposu, GorevGirdi};

    fn depo() -> GorevDeposu {
        GorevDeposu::yeni()
    }

    fn ekle(depo: &mut GorevDeposu, baslik: &str, dakika: i32, oncelik: Oncelik) -> u32 {
        depo.ekle(
            GorevGirdi {
                baslik: baslik.to_string(),
                tahmini_dakika: dakika,
                oncelik,
                etiketler: Vec::new(),
                bagimliliklar: Vec::new(),
            },
            1_000,
        )
        .unwrap()
    }

    fn kapasite(kap: i32, sabit: i32) -> GunlukKapasite {
        GunlukKapasite::yeni("2026-09-29", kap, sabit).unwrap()
    }

    #[test]
    fn negatif_kapasite_reddedilir() {
        assert!(GunlukKapasite::yeni("2026-09-29", -1, 0).is_err());
        assert!(GunlukKapasite::yeni("2026-09-29", 10, -5).is_err());
    }

    #[test]
    fn sabit_sure_kapasiteden_dusulur() {
        let k = kapasite(180, 60);
        assert_eq!(k.kullanilabilir_dakika(), 120);
        // Sabit sure kapasiteyi asarsa kullanilabilir sifir, negatife dusmez.
        let k = kapasite(30, 90);
        assert_eq!(k.kullanilabilir_dakika(), 0);
    }

    #[test]
    fn kapasiteye_sigan_gorevler_plana_girer() {
        let mut d = depo();
        ekle(&mut d, "A", 50, Oncelik::Yuksek);
        ekle(&mut d, "B", 40, Oncelik::Normal);
        ekle(&mut d, "C", 20, Oncelik::Normal);
        let plan = planla(&d, &kapasite(120, 0)).unwrap();
        assert_eq!(plan.sigacaklar.len(), 3);
        assert_eq!(plan.kullanilan_dakika, 110);
        assert_eq!(plan.kalan_dakika(), 10);
        assert!(plan.uyarilar.is_empty());
    }

    #[test]
    fn kapasiteyi_asan_gorev_isaretlenir_ve_uyari_uretir() {
        let mut d = depo();
        ekle(&mut d, "A", 100, Oncelik::Yuksek);
        ekle(&mut d, "B", 60, Oncelik::Dusuk);
        let plan = planla(&d, &kapasite(120, 0)).unwrap();
        assert_eq!(plan.sigacaklar.len(), 1);
        assert_eq!(plan.sigmayanlar.len(), 1);
        let sigmayan = &plan.sigmayanlar[0];
        assert_eq!(sigmayan.sebep, Some(SigmaSebebi::Kapasite));
        assert!(
            sigmayan.aciklama.contains("kalan"),
            "aciklama: {}",
            sigmayan.aciklama
        );
        assert_eq!(plan.uyarilar.len(), 1, "kapasite asimi uyarisi");
        assert!(
            plan.uyarilar[0].contains("2 sa 40 dk"),
            "uyari: {:?}",
            plan.uyarilar
        );
        assert!(!plan.uyarilar[0].contains("dk dk"), "cift birim olmamali");
    }

    #[test]
    fn oncelik_kapasiteyi_once_dagitir() {
        let mut d = depo();
        ekle(&mut d, "dusuk", 50, Oncelik::Dusuk);
        ekle(&mut d, "yuksek", 50, Oncelik::Yuksek);
        ekle(&mut d, "normal", 50, Oncelik::Normal);
        let plan = planla(&d, &kapasite(100, 0)).unwrap();
        let siralanmis: Vec<u32> = plan.sigacaklar.iter().map(|o| o.gorev_id).collect();
        assert_eq!(siralanmis, vec![2, 3]);
        assert_eq!(plan.sigmayanlar[0].sebep, Some(SigmaSebebi::Kapasite));
    }

    #[test]
    fn bagimlilik_tamamlanmadan_gorev_plana_girmez() {
        let mut d = depo();
        let onay = ekle(&mut d, "Musteri onayi", 30, Oncelik::Yuksek);
        let duzeltme = ekle(&mut d, "Yayin oncesi duzeltme", 45, Oncelik::Yuksek);
        d.bagimlilik_ekle(duzeltme, onay).unwrap();
        let plan = planla(&d, &kapasite(240, 0)).unwrap();
        assert_eq!(plan.sigacaklar.len(), 1);
        assert_eq!(plan.sigmayanlar.len(), 1);
        assert_eq!(plan.sigmayanlar[0].sebep, Some(SigmaSebebi::Bagimlilik));
        assert!(plan.sigmayanlar[0].aciklama.contains("bekliyor"));
        // Bagimlilik tamamlaninca gorev plana girer.
        d.durum_degistir(onay, Durum::Tamamlandi, 2_000).unwrap();
        let plan = planla(&d, &kapasite(240, 0)).unwrap();
        assert_eq!(plan.sigacaklar.len(), 1);
        assert_eq!(plan.sigacaklar[0].gorev_id, duzeltme);
    }

    #[test]
    fn ertelenmis_bagimlilik_plani_engeller() {
        let mut d = depo();
        let onay = ekle(&mut d, "Onay", 30, Oncelik::Yuksek);
        let duzeltme = ekle(&mut d, "Duzeltme", 45, Oncelik::Yuksek);
        d.bagimlilik_ekle(duzeltme, onay).unwrap();
        d.durum_degistir(onay, Durum::Ertelendi, 2_000).unwrap();
        let plan = planla(&d, &kapasite(240, 0)).unwrap();
        assert_eq!(plan.sigmayanlar.len(), 2);
        let duzeltme_ogesi = plan
            .sigmayanlar
            .iter()
            .find(|o| o.gorev_id == duzeltme)
            .unwrap();
        assert_eq!(duzeltme_ogesi.sebep, Some(SigmaSebebi::Bagimlilik));
        assert!(duzeltme_ogesi.aciklama.contains("ertelendi"));
    }

    #[test]
    fn tamamlanan_gorev_plana_girmez() {
        let mut d = depo();
        let a = ekle(&mut d, "Bitti", 30, Oncelik::Yuksek);
        ekle(&mut d, "Yapilacak", 30, Oncelik::Normal);
        d.durum_degistir(a, Durum::Tamamlandi, 2_000).unwrap();
        let plan = planla(&d, &kapasite(120, 0)).unwrap();
        assert_eq!(plan.sigacaklar.len(), 1);
        assert!(plan.sigmayanlar.is_empty());
    }

    #[test]
    fn calisan_ve_ertelenen_gorevler_plana_girmez() {
        let mut d = depo();
        let a = ekle(&mut d, "Calisiyor", 30, Oncelik::Yuksek);
        let b = ekle(&mut d, "Ertelendi", 30, Oncelik::Yuksek);
        d.durum_degistir(a, Durum::Calisiyor, 2_000).unwrap();
        d.durum_degistir(b, Durum::Ertelendi, 2_000).unwrap();
        let plan = planla(&d, &kapasite(120, 0)).unwrap();
        assert!(plan.sigacaklar.is_empty());
        let sebepler: Vec<SigmaSebebi> = plan.sigmayanlar.iter().filter_map(|o| o.sebep).collect();
        assert!(sebepler.contains(&SigmaSebebi::Calisiyor));
        assert!(sebepler.contains(&SigmaSebebi::Ertelendi));
    }

    #[test]
    fn tahmini_sure_sifir_olan_gorev_plana_girmez() {
        let mut d = depo();
        ekle(&mut d, "Belirsiz", 0, Oncelik::Yuksek);
        let plan = planla(&d, &kapasite(120, 0)).unwrap();
        assert!(plan.sigacaklar.is_empty());
        assert_eq!(plan.sigmayanlar[0].sebep, Some(SigmaSebebi::TahminYok));
    }

    #[test]
    fn sifir_kapasite_uyari_uretir() {
        let mut d = depo();
        ekle(&mut d, "A", 30, Oncelik::Yuksek);
        let plan = planla(&d, &kapasite(30, 30)).unwrap();
        assert_eq!(plan.kullanilabilir_dakika, 0);
        assert!(plan.sigacaklar.is_empty());
        assert_eq!(plan.uyarilar.len(), 1);
        assert!(plan.uyarilar[0].contains("sifir"));
    }

    #[test]
    fn bos_depo_bos_plan_uretir() {
        let d = depo();
        let plan = planla(&d, &kapasite(120, 0)).unwrap();
        assert!(plan.sigacaklar.is_empty());
        assert!(plan.sigmayanlar.is_empty());
        assert!(plan.uyarilar.is_empty());
        assert_eq!(plan.kalan_dakika(), 120);
    }

    #[test]
    fn plan_metni_gerekceleri_gosterir() {
        let mut d = depo();
        let onay = ekle(&mut d, "Onay", 30, Oncelik::Yuksek);
        let duzeltme = ekle(&mut d, "Duzeltme", 45, Oncelik::Yuksek);
        d.bagimlilik_ekle(duzeltme, onay).unwrap();
        let plan = planla(&d, &kapasite(240, 0)).unwrap();
        let metin = plan.metin();
        assert!(metin.contains("BUGUN SIGANLAR"), "{metin}");
        assert!(metin.contains("BUGUN SIGMAYANLAR"), "{metin}");
        assert!(metin.contains("[bagimlilik]"), "{metin}");
        assert!(metin.contains("#1 (bekliyor)"), "{metin}");
        assert!(!metin.contains("dk dk"), "cift birim olmamali: {metin}");
    }
}
