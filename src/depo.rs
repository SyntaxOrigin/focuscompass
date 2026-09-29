//! Kalici depolama: surumlu gorev dosyasi, satir satir oturum gunlugu,
//! gunluk Markdown dosyalari ve haftalik raporlar.
//!
//! Yazma **atomiktir**: once `hedef.tmp` yazilir, sonra `rename` ile yerine
//! gecer. Guc kesintisi yalnizca gecici dosyayi bozar; asil dosya ya eski
//! icerigini korur ya da yenisini butun olarak gosterir. Gunluk ise JSONL
//! olarak **eklenir**; her oturum tek satirdir ve tam dosya yeniden yazilmaz.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::gorev::{Gorev, GorevDeposu};
use crate::hata::{Hata, Sonuc};
use crate::oturum::Oturum;

/// `gorevler.json` dosyasinin sema surumu.
///
/// Bu surum artirildiginda eski dosyalar okunamaz; arac reddeder, dosyayi
/// **silmez ve degistirmez** (kullanici verisi korunur).
pub const SEMA_SURUMU: u32 = 1;

/// `gorevler.json` dosyasinin govdesi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GorevDosyasi {
    /// Seman surumu.
    pub surum: u32,
    /// Dosyanin son yazildigi zaman (UTC epoch saniye).
    pub guncelleme: i64,
    /// Gorev listesi.
    pub gorevler: Vec<Gorev>,
}

impl GorevDosyasi {
    /// Yeni bir dosya govdesi olusturur.
    pub fn bos(simdi: i64) -> Self {
        Self {
            surum: SEMA_SURUMU,
            guncelleme: simdi,
            gorevler: Vec::new(),
        }
    }
}

/// Veri klasoru uzerindeki tum dosya islemlerini yoneten depo.
#[derive(Debug, Clone)]
pub struct Depo {
    kok: PathBuf,
}

impl Depo {
    /// Veri klasorunu tanimlar. Klasorun var olmasi gerekmez.
    pub fn yeni(kok: impl Into<PathBuf>) -> Self {
        Self { kok: kok.into() }
    }

    /// Veri klasorunun tam yolu.
    pub fn kok(&self) -> &Path {
        &self.kok
    }

    /// Gorev dosyasinin tam yolu.
    pub fn gorev_yolu(&self) -> PathBuf {
        self.kok.join("gorevler.json")
    }

    /// Oturum gunlugu dosyasinin tam yolu.
    pub fn oturum_yolu(&self) -> PathBuf {
        self.kok.join("oturumlar.jsonl")
    }

    /// Gunluk Markdown dosyasinin tam yolu.
    pub fn gunluk_yolu(&self, tarih: &str) -> PathBuf {
        self.kok.join("gunluk").join(format!("{tarih}.md"))
    }

    /// Rapor dosyasinin tam yolu.
    pub fn rapor_yolu(&self, ad: &str) -> PathBuf {
        self.kok.join("raporlar").join(ad)
    }

    /// Veri klasorunu ve alt klasorlerini olusturur; var olanlara dokunmaz.
    pub fn hazirla(&self) -> Sonuc<()> {
        for dizin in [
            self.kok.clone(),
            self.kok.join("gunluk"),
            self.kok.join("raporlar"),
        ] {
            std::fs::create_dir_all(&dizin).map_err(|hata| Hata::Dosya {
                yol: dizin,
                ayrinti: hata.to_string(),
            })?;
        }
        Ok(())
    }

    /// Gorev dosyasini okur.
    ///
    /// Dosya yoksa bos depo doner (ilk kullanim). Surum uyusmazligi ve bozuk
    /// JSON ayri hata turleridir; hicbir durumda dosya **silinmez**.
    pub fn gorevleri_oku(&self) -> Sonuc<GorevDeposu> {
        let yol = self.gorev_yolu();
        if !yol.exists() {
            return Ok(GorevDeposu::yeni());
        }
        let metin = std::fs::read_to_string(&yol).map_err(|hata| Hata::Dosya {
            yol: yol.clone(),
            ayrinti: hata.to_string(),
        })?;
        let govde = parse_gorev_dosyasi(&metin)?;
        let depo = GorevDeposu::gorevlerden(govde.gorevler)?;
        depo.denetle()?;
        Ok(depo)
    }

    /// Gorev dosyasini atomik olarak yazar.
    pub fn gorevleri_yaz(&self, depo: &GorevDeposu, simdi: i64) -> Sonuc<()> {
        let govde = GorevDosyasi {
            surum: SEMA_SURUMU,
            guncelleme: simdi,
            gorevler: depo.tum().to_vec(),
        };
        let metin = serde_json::to_string_pretty(&govde).map_err(|hata| Hata::BozukJson {
            ayrinti: hata.to_string(),
        })?;
        atomik_yaz(&self.gorev_yolu(), &metin)
    }

    /// Oturum gunlugunu okur; dosya yoksa bos gunluk doner.
    ///
    /// Tek bir bozuk satir tum gunlugu reddeder ve satir numarasi hatada verilir;
    /// bu, sessiz veri kaybini onlemek icindir.
    pub fn oturumlari_oku(&self) -> Sonuc<Vec<Oturum>> {
        let yol = self.oturum_yolu();
        if !yol.exists() {
            return Ok(Vec::new());
        }
        let metin = std::fs::read_to_string(&yol).map_err(|hata| Hata::Dosya {
            yol: yol.clone(),
            ayrinti: hata.to_string(),
        })?;
        oturumlari_coz(&metin, &yol)
    }

    /// Oturum gunluguna tek satir ekler (JSONL, eklenebilir ve kirilmaz).
    pub fn oturum_ekle(&self, oturum: &Oturum) -> Sonuc<()> {
        let metin = serde_json::to_string(oturum).map_err(|hata| Hata::BozukJson {
            ayrinti: hata.to_string(),
        })?;
        satir_ekle(&self.oturum_yolu(), &metin)
    }

    /// Gunluk Markdown dosyasini atomik olarak yazar.
    pub fn gunluk_yaz(&self, tarih: &str, icerik: &str) -> Sonuc<()> {
        atomik_yaz(&self.gunluk_yolu(tarih), icerik)
    }

    /// Gunluk Markdown dosyasini okur; yoksa bos metin doner.
    pub fn gunluk_oku(&self, tarih: &str) -> Sonuc<String> {
        let yol = self.gunluk_yolu(tarih);
        if !yol.exists() {
            return Ok(String::new());
        }
        std::fs::read_to_string(&yol).map_err(|hata| Hata::Dosya {
            yol,
            ayrinti: hata.to_string(),
        })
    }

    /// Rapor dosyasini atomik olarak yazar.
    pub fn rapor_yaz(&self, ad: &str, icerik: &str) -> Sonuc<()> {
        atomik_yaz(&self.rapor_yolu(ad), icerik)
    }
}

/// Gecici dosya yolunu uretir: `gorevler.json` -> `gorevler.json.tmp`.
pub fn gecici_yol(yol: &Path) -> PathBuf {
    let mut ad = yol.as_os_str().to_os_string();
    ad.push(".tmp");
    PathBuf::from(ad)
}

/// Icerigi hedef dosyaya **atomik** olarak yazar.
///
/// Sirasi: gecici dosyaya yaz -> icerigi diske zorla (`sync_all`) -> `rename`.
/// Gecici dosya yazilamazsa veya `rename` basarisiz olursa gecici dosya
/// silinir ve **hedef dosya hic degistirilmez**.
pub fn atomik_yaz(yol: &Path, icerik: &str) -> Sonuc<()> {
    if let Some(dizin) = yol.parent() {
        if !dizin.as_os_str().is_empty() {
            std::fs::create_dir_all(dizin).map_err(|hata| Hata::Dosya {
                yol: dizin.to_path_buf(),
                ayrinti: hata.to_string(),
            })?;
        }
    }
    let gecici = gecici_yol(yol);
    let yazma = (|| -> std::io::Result<()> {
        let mut dosya = File::create(&gecici)?;
        dosya.write_all(icerik.as_bytes())?;
        dosya.flush()?;
        dosya.sync_all()?;
        drop(dosya);
        std::fs::rename(&gecici, yol)
    })();
    if let Err(hata) = yazma {
        // Yarim gecici dosya kalmasin. Temizlik basarisiz olabilir (ornegin
        // gecici yol bir dizinse); bu durumda sessizce yutulur, asil hata doner.
        let _ = std::fs::remove_file(&gecici);
        return Err(Hata::Dosya {
            yol: yol.to_path_buf(),
            ayrinti: hata.to_string(),
        });
    }
    Ok(())
}

/// JSONL dosyasina tek satir ekler; dosya yoksa olusturulur.
pub fn satir_ekle(yol: &Path, satir: &str) -> Sonuc<()> {
    if let Some(dizin) = yol.parent() {
        if !dizin.as_os_str().is_empty() {
            std::fs::create_dir_all(dizin).map_err(|hata| Hata::Dosya {
                yol: dizin.to_path_buf(),
                ayrinti: hata.to_string(),
            })?;
        }
    }
    let mut dosya = OpenOptions::new()
        .create(true)
        .append(true)
        .open(yol)
        .map_err(|hata| Hata::Dosya {
            yol: yol.to_path_buf(),
            ayrinti: hata.to_string(),
        })?;
    let mut kayit = String::with_capacity(satir.len() + 1);
    kayit.push_str(satir);
    kayit.push('\n');
    dosya
        .write_all(kayit.as_bytes())
        .map_err(|hata| Hata::Dosya {
            yol: yol.to_path_buf(),
            ayrinti: hata.to_string(),
        })?;
    dosya.flush().map_err(|hata| Hata::Dosya {
        yol: yol.to_path_buf(),
        ayrinti: hata.to_string(),
    })
}

/// Gorev dosyasi metnini cozer; surum ve JSON hatalarini ayirir.
fn parse_gorev_dosyasi(metin: &str) -> Sonuc<GorevDosyasi> {
    let deger: serde_json::Value = serde_json::from_str(metin).map_err(|hata| Hata::BozukJson {
        ayrinti: hata.to_string(),
    })?;
    match deger.get("surum").and_then(serde_json::Value::as_u64) {
        Some(surum) => {
            let surum = u32::try_from(surum).map_err(|_| Hata::SurumUyusmazligi {
                bulunan: 0,
                beklenen: SEMA_SURUMU,
            })?;
            if surum != SEMA_SURUMU {
                return Err(Hata::SurumUyusmazligi {
                    bulunan: surum,
                    beklenen: SEMA_SURUMU,
                });
            }
        }
        None => {
            return Err(Hata::BozukJson {
                ayrinti: format!("'surum' alani eksik (beklenen: {SEMA_SURUMU})"),
            })
        }
    }
    serde_json::from_value(deger).map_err(|hata| Hata::BozukJson {
        ayrinti: hata.to_string(),
    })
}

/// JSONL metnini oturum listesine cozer; hatali satir numarasiyla bildirilir.
fn oturumlari_coz(metin: &str, yol: &Path) -> Sonuc<Vec<Oturum>> {
    let mut oturumlar = Vec::new();
    for (indeks, satir) in metin.lines().enumerate() {
        if satir.trim().is_empty() {
            continue;
        }
        let oturum: Oturum = serde_json::from_str(satir).map_err(|hata| Hata::BozukSatir {
            yol: yol.to_path_buf(),
            satir_no: indeks + 1,
            ayrinti: hata.to_string(),
        })?;
        oturumlar.push(oturum);
    }
    Ok(oturumlar)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use crate::gorev::{GorevGirdi, Oncelik};
    use crate::zaman::SaatDilimi;

    /// Gecici dizin ureten, Drop ile temizleyen kapsayici.
    ///
    /// `tempfile` crate'i bagimlilik politikasiyla yasaktir; yardimci kendi
    /// kodumuzla yazildi. Benzersizlik etiket + surec kimliginden gelir.
    struct GeciciDizin {
        yol: PathBuf,
    }

    impl GeciciDizin {
        fn yeni(etiket: &str) -> Self {
            let kok = std::env::temp_dir().join(format!("fc-depo-{etiket}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&kok);
            std::fs::create_dir_all(&kok).unwrap();
            Self { yol: kok }
        }

        fn yol(&self) -> &Path {
            &self.yol
        }
    }

    impl Drop for GeciciDizin {
        fn drop(&mut self) {
            // Drop icinden hata dondurulemez; temizlik basarisiz olsa da testi dusurmemeli.
            let _ = std::fs::remove_dir_all(&self.yol);
        }
    }

    /// Ornek veri: tek bir gorev olusturur ve **diske yazar**.
    fn depo_ornegi(kok: &Path) -> (Depo, u32) {
        let depo = Depo::yeni(kok);
        depo.hazirla().unwrap();
        let mut d = GorevDeposu::yeni();
        let id = d
            .ekle(
                GorevGirdi {
                    baslik: "Rapor taslagi".to_string(),
                    tahmini_dakika: 45,
                    oncelik: Oncelik::Yuksek,
                    etiketler: vec!["rapor".to_string()],
                    bagimliliklar: Vec::new(),
                },
                1_000,
            )
            .unwrap();
        depo.gorevleri_yaz(&d, 1_000).unwrap();
        (depo, id)
    }

    #[test]
    fn gorevleri_yaz_ve_oku_gidis_donus_yapar() {
        let gecici = GeciciDizin::yeni("gidis-donus");
        let (depo, id) = depo_ornegi(gecici.yol());
        let mut d = GorevDeposu::gorevlerden(depo.gorevleri_oku().unwrap().tum().to_vec()).unwrap();
        let id2 = d
            .ekle(
                GorevGirdi {
                    baslik: "Ikinci".to_string(),
                    tahmini_dakika: 20,
                    oncelik: Oncelik::Normal,
                    etiketler: Vec::new(),
                    bagimliliklar: Vec::new(),
                },
                2_000,
            )
            .unwrap();
        d.bagimlilik_ekle(id2, id).unwrap();
        depo.gorevleri_yaz(&d, 3_000).unwrap();

        let okunan = depo.gorevleri_oku().unwrap();
        assert_eq!(okunan.uzunluk(), 2);
        assert_eq!(okunan.getir(id2).unwrap().bagimliliklar, vec![id]);
        assert_eq!(okunan.getir(id).unwrap().etiketler, vec!["rapor"]);
    }

    #[test]
    fn surum_alani_kaydedilir() {
        let gecici = GeciciDizin::yeni("surum");
        let (depo, _) = depo_ornegi(gecici.yol());
        let d = GorevDeposu::yeni();
        depo.gorevleri_yaz(&d, 9_000).unwrap();
        let metin = std::fs::read_to_string(depo.gorev_yolu()).unwrap();
        let deger: serde_json::Value = serde_json::from_str(&metin).unwrap();
        assert_eq!(deger["surum"], SEMA_SURUMU);
        assert_eq!(deger["guncelleme"], 9_000);
    }

    #[test]
    fn atomik_yazma_yarim_dosya_birakmaz() {
        let gecici = GeciciDizin::yeni("atomik");
        let hedef = gecici.yol().join("gorevler.json");
        atomik_yaz(&hedef, "{\"surum\":1,\"eski\":true}").unwrap();

        // Gecici yolun yerine bir dizin yerlestirilerek yazma zorunlu hata yaratilir.
        let gecici_yol_deger = gecici_yol(&hedef);
        std::fs::create_dir(&gecici_yol_deger).unwrap();

        let sonuc = atomik_yaz(&hedef, "{\"surum\":1,\"yeni\":true}");
        assert!(sonuc.is_err(), "yazma basarisiz olmaliydi");

        let sonra = std::fs::read_to_string(&hedef).unwrap();
        assert_eq!(sonra, "{\"surum\":1,\"eski\":true}");
        assert!(!sonra.contains("yeni"), "hedef dosya hic degismemis olmali");
    }

    #[test]
    fn basarili_yazma_gecici_dosya_artigi_birakmaz() {
        let gecici = GeciciDizin::yeni("artik");
        let hedef = gecici.yol().join("rapor.md");
        atomik_yaz(&hedef, "# Baslik\n").unwrap();
        assert!(hedef.exists());
        assert!(!gecici_yol(&hedef).exists(), "gecici dosya kalmamali");
        assert_eq!(std::fs::read_to_string(&hedef).unwrap(), "# Baslik\n");
    }

    #[test]
    fn atomik_yazma_eksik_klasoru_olusturur() {
        let gecici = GeciciDizin::yeni("klasor");
        let hedef = gecici.yol().join("a").join("b").join("x.json");
        atomik_yaz(&hedef, "{}").unwrap();
        assert!(hedef.exists());
    }

    #[test]
    fn bozuk_json_reddedilir() {
        let gecici = GeciciDizin::yeni("bozuk");
        let (depo, _) = depo_ornegi(gecici.yol());
        std::fs::write(depo.gorev_yolu(), "{\"surum\":1,\"gorevler\":[").unwrap();
        let sonuc = depo.gorevleri_oku();
        assert!(
            matches!(sonuc, Err(Hata::BozukJson { .. })),
            "gelen: {sonuc:?}"
        );
    }

    #[test]
    fn surum_uyusmazligi_reddedilir() {
        let gecici = GeciciDizin::yeni("uyusmazlik");
        let (depo, _) = depo_ornegi(gecici.yol());
        std::fs::write(depo.gorev_yolu(), "{\"surum\":99,\"gorevler\":[]}").unwrap();
        let sonuc = depo.gorevleri_oku();
        match sonuc {
            Err(Hata::SurumUyusmazligi { bulunan, beklenen }) => {
                assert_eq!(bulunan, 99);
                assert_eq!(beklenen, SEMA_SURUMU);
            }
            diger => panic!("surum hatasi bekleniyordu, gelen: {diger:?}"),
        }
    }

    #[test]
    fn surum_alani_eksikse_bozuk_json_sayilir() {
        let gecici = GeciciDizin::yeni("eksik-surum");
        let (depo, _) = depo_ornegi(gecici.yol());
        std::fs::write(depo.gorev_yolu(), "{\"gorevler\":[]}").unwrap();
        assert!(matches!(depo.gorevleri_oku(), Err(Hata::BozukJson { .. })));
    }

    #[test]
    fn dosya_yoksa_bos_depo_doner() {
        let gecici = GeciciDizin::yeni("bos");
        let depo = Depo::yeni(gecici.yol().join("yok"));
        let d = depo.gorevleri_oku().unwrap();
        assert!(d.bos_mu());
        assert!(depo.oturumlari_oku().unwrap().is_empty());
    }

    #[test]
    fn elle_bozulan_bagimlilik_denetlemede_yakalanir() {
        let gecici = GeciciDizin::yeni("asiili");
        let (depo, _) = depo_ornegi(gecici.yol());
        let metin = "{\"surum\":1,\"guncelleme\":1,\"gorevler\":[{\"id\":1,\
\"baslik\":\"X\",\"tahmini_dakika\":10,\"oncelik\":\"normal\",\"durum\":\"bekliyor\",\
\"etiketler\":[],\"bagimliliklar\":[42],\"olusturma\":1,\"guncelleme\":1}]}";
        std::fs::write(depo.gorev_yolu(), metin).unwrap();
        assert!(matches!(
            depo.gorevleri_oku(),
            Err(Hata::BilinmeyenBagimlilik { .. })
        ));
    }

    #[test]
    fn elle_yazilan_dongu_denetlemede_yakalanir() {
        let gecici = GeciciDizin::yeni("dongu");
        let (depo, _) = depo_ornegi(gecici.yol());
        let metin = "{\"surum\":1,\"guncelleme\":1,\"gorevler\":[\
{\"id\":1,\"baslik\":\"A\",\"tahmini_dakika\":10,\"oncelik\":\"normal\",\
\"durum\":\"bekliyor\",\"etiketler\":[],\"bagimliliklar\":[2],\"olusturma\":1,\"guncelleme\":1},\
{\"id\":2,\"baslik\":\"B\",\"tahmini_dakika\":10,\"oncelik\":\"normal\",\
\"durum\":\"bekliyor\",\"etiketler\":[],\"bagimliliklar\":[1],\"olusturma\":1,\"guncelleme\":1}]}";
        std::fs::write(depo.gorev_yolu(), metin).unwrap();
        assert!(matches!(
            depo.gorevleri_oku(),
            Err(Hata::BagimlilikDongusu { .. })
        ));
    }

    #[test]
    fn oturum_jsonl_satir_satir_eklenir() {
        let gecici = GeciciDizin::yeni("jsonl");
        let (depo, _) = depo_ornegi(gecici.yol());
        for sira in 1..=3 {
            let mut o = Oturum::yeni(
                sira,
                Some(1),
                crate::oturum::OturumTuru::Odak,
                25,
                sira as i64,
            )
            .unwrap();
            o.bitir(i64::from(sira) * 25 * 60).unwrap();
            depo.oturum_ekle(&o).unwrap();
        }
        let oturumlar = depo.oturumlari_oku().unwrap();
        assert_eq!(oturumlar.len(), 3);
        let satir_sayisi = std::fs::read_to_string(depo.oturum_yolu())
            .unwrap()
            .lines()
            .count();
        assert_eq!(satir_sayisi, 3);
    }

    #[test]
    fn oturum_jsonl_bozuk_satir_hata_verir() {
        let gecici = GeciciDizin::yeni("jsonl-bozuk");
        let (depo, _) = depo_ornegi(gecici.yol());
        let gecerli = Oturum::yeni(1, Some(1), crate::oturum::OturumTuru::Odak, 25, 1_000).unwrap();
        let metin = serde_json::to_string(&gecerli).unwrap();
        satir_ekle(&depo.oturum_yolu(), &metin).unwrap();
        satir_ekle(&depo.oturum_yolu(), "bozuk satir").unwrap();
        match depo.oturumlari_oku() {
            Err(Hata::BozukSatir { satir_no, .. }) => assert_eq!(satir_no, 2),
            diger => panic!("bozuk satir hatasi bekleniyordu, gelen: {diger:?}"),
        }
    }

    #[test]
    fn gunluk_markdown_yazilir_ve_oku() {
        let gecici = GeciciDizin::yeni("gunluk");
        let (depo, _) = depo_ornegi(gecici.yol());
        let tarih = crate::zaman::tarih_yaz(crate::zaman::SaatDilimi::yeni(180).unwrap().gun_no(0));
        assert_eq!(depo.gunluk_oku(&tarih).unwrap(), "");
        depo.gunluk_yaz(&tarih, "# Gunluk\n").unwrap();
        assert_eq!(depo.gunluk_oku(&tarih).unwrap(), "# Gunluk\n");
        assert!(depo.gunluk_yolu(&tarih).exists());
    }

    #[test]
    fn gunluk_dosya_yolu_tarihle_eslesir() {
        let gecici = GeciciDizin::yeni("yol");
        let depo = Depo::yeni(gecici.yol());
        let yol = depo.gunluk_yolu("2026-09-29");
        assert!(
            yol.to_string_lossy().ends_with("gunluk\\2026-09-29.md")
                || yol.to_string_lossy().ends_with("gunluk/2026-09-29.md")
        );
    }

    #[test]
    fn hazirla_alt_klasorleri_olusturur() {
        let gecici = GeciciDizin::yeni("hazirla");
        let depo = Depo::yeni(gecici.yol().join("veri"));
        depo.hazirla().unwrap();
        assert!(depo.gunluk_yolu("x").parent().unwrap().exists());
        assert!(depo.rapor_yolu("x.md").parent().unwrap().exists());
        // Ikinci cagri hata vermemeli.
        depo.hazirla().unwrap();
    }

    #[test]
    fn saat_dilimi_bagimsiz_okuma() {
        // Ayni veri farkli saat dilimiyle ayni gun sayisini vermeli (dosya epoch tutar).
        let gecici = GeciciDizin::yeni("dilim");
        let (depo, id) = depo_ornegi(gecici.yol());
        let d = GorevDeposu::gorevlerden(depo.gorevleri_oku().unwrap().tum().to_vec()).unwrap();
        assert_eq!(d.getir(id).unwrap().olusturma, 1_000);
        let _ = SaatDilimi::yeni(0).unwrap();
    }
}
