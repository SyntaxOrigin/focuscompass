//! Pomodoro oturumu, oturum gecmisi ve kesinti gunlugu.
//!
//! Sure yalnizca `std::time` ile olculur. Oturum sureleri **duvar saati farkindan
//! bagimsiz** kalabilsin diye epoch farki hesaplanir; isletim sistemi saatini
//! degistiren bir islem oturum suresini bozamaz. Ayni anda yalnizca bir oturum
//! acik olabilir: ust uste pomodoro kural burada zorlanir.

use serde::{Deserialize, Serialize};

use crate::hata::{Hata, Sonuc};

/// Oturumun cesidi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OturumTuru {
    /// Derin calisma odagi.
    Odak,
    /// Oturumlar arasinda kisa mola.
    KisaMola,
    /// Her dort oturumdan sonra uzun mola.
    UzunMola,
}

impl OturumTuru {
    /// ASCII ad.
    pub fn ad(self) -> &'static str {
        match self {
            Self::Odak => "odak",
            Self::KisaMola => "kisa-mola",
            Self::UzunMola => "uzun-mola",
        }
    }

    /// ASCII adindan tur cozer.
    pub fn ayir(metin: &str) -> Sonuc<Self> {
        match metin {
            "odak" | "focus" => Ok(Self::Odak),
            "kisa-mola" | "kisa" | "short" => Ok(Self::KisaMola),
            "uzun-mola" | "uzun" | "long" => Ok(Self::UzunMola),
            _ => Err(Hata::GecersizFormat {
                girdi: metin.to_string(),
            }),
        }
    }

    /// Bu turun odak sayilip sayilmayacagi.
    pub fn odak_mi(self) -> bool {
        matches!(self, Self::Odak)
    }
}

/// Oturumun kapanma sekli.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OturumDurumu {
    /// Planlanan sureyi doldurdu.
    Tamamlandi,
    /// Kullanici oturumu erken kapatti.
    Birakildi,
}

impl OturumDurumu {
    /// ASCII ad.
    pub fn ad(self) -> &'static str {
        match self {
            Self::Tamamlandi => "tamamlandi",
            Self::Birakildi => "birakildi",
        }
    }
}

/// Bir oturum icinde kaydedilen tek bir kesinti.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kesinti {
    /// Kesintinin kaynagi (kisa metin, serbest metin alani kullaniciya acilmaz).
    pub kaynak: String,
    /// Kesintinin suresi (dakika).
    pub dakika: i32,
}

/// Tek bir pomodoro oturumu kaydi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Oturum {
    /// Oturumun gunluk icindeki sira numarasi (1'den baslar).
    pub sira: u32,
    /// Oturumun bagli oldugu gorev; mola oturumlerinde `None`.
    pub gorev_id: Option<u32>,
    /// Oturum turu.
    pub tur: OturumTuru,
    /// Kapanma sekli.
    pub durum: OturumDurumu,
    /// Baslangic zamani (UTC epoch saniye).
    pub baslangic: i64,
    /// Bitis zamani; kapali degilse `None`.
    pub bitis: Option<i64>,
    /// Planlanan sure (dakika).
    pub planli_dakika: i32,
    /// Oturum icindeki kesintiler.
    pub kesintiler: Vec<Kesinti>,
}

impl Oturum {
    /// Yeni bir oturum baslatir ve planlanan sureyi dogrular.
    ///
    /// Sifir ve negatif sure reddedilir; pomodoro suresi sifir olamaz.
    pub fn yeni(
        sira: u32,
        gorev_id: Option<u32>,
        tur: OturumTuru,
        planli_dakika: i32,
        baslangic: i64,
    ) -> Sonuc<Self> {
        if planli_dakika < 0 {
            return Err(Hata::NegatifSure {
                deger: planli_dakika,
            });
        }
        if planli_dakika == 0 {
            return Err(Hata::SifirSure);
        }
        Ok(Self {
            sira,
            gorev_id,
            tur,
            durum: OturumDurumu::Tamamlandi,
            baslangic,
            bitis: None,
            planli_dakika,
            kesintiler: Vec::new(),
        })
    }

    /// Oturum kapandi mi.
    pub fn kapali(&self) -> bool {
        self.bitis.is_some()
    }

    /// Oturumu kapatir; bitis baslangictan once olamaz.
    ///
    /// Planlanan sureyi doldurduysa `Tamamlandi`, dolmadiysa `Birakildi`
    /// olarak isaretlenir. Yarim kalan oturum ayrica bir durum olarak raporlanir.
    pub fn bitir(&mut self, bitis: i64) -> Sonuc<()> {
        if self.kapali() {
            return Err(Hata::OturumKapatilmis { sira: self.sira });
        }
        if bitis < self.baslangic {
            return Err(Hata::BitisBaslangictanOnce { sira: self.sira });
        }
        self.durum = if self.gecen_dakika(bitis) >= i64::from(self.planli_dakika) {
            OturumDurumu::Tamamlandi
        } else {
            OturumDurumu::Birakildi
        };
        self.bitis = Some(bitis);
        Ok(())
    }

    /// Verilen bitis anina kadar gecen **gercek** sure (dakika, asagi yuvarlanmis).
    pub fn gecen_dakika(&self, bitis: i64) -> i64 {
        (bitis - self.baslangic) / 60
    }

    /// Kapanmis oturumun toplam gecen suresi (dakika).
    pub fn gecen_dakika_toplam(&self) -> Sonuc<i64> {
        match self.bitis {
            Some(bitis) => Ok(self.gecen_dakika(bitis)),
            None => Err(Hata::AcikOturumYok),
        }
    }

    /// Kesintilerin toplam suresi (dakika).
    pub fn toplam_kesinti(&self) -> i32 {
        self.kesintiler.iter().map(|k| k.dakika).sum()
    }

    /// Oturuma kesinti ekler.
    ///
    /// Kesinti suresi pozitif olmalidir ve kesintilerin toplami oturumdan gecen
    /// sureyi asmamalidir; aksi halde kayit reddedilir (ozel "odak suresi" olusmaz).
    pub fn kesinti_ekle(&mut self, kaynak: &str, dakika: i32, gecen_dakika: i32) -> Sonuc<()> {
        if dakika < 0 {
            return Err(Hata::NegatifSure { deger: dakika });
        }
        if dakika == 0 {
            return Err(Hata::SifirSure);
        }
        let kaynak = kaynak.trim();
        if kaynak.is_empty() {
            return Err(Hata::BosBaslik);
        }
        let toplam = self.toplam_kesinti() + dakika;
        if toplam > gecen_dakika {
            return Err(Hata::KesintiAsim {
                toplam,
                gecen: gecen_dakika,
            });
        }
        self.kesintiler.push(Kesinti {
            kaynak: kaynak.to_string(),
            dakika,
        });
        Ok(())
    }

    /// Kapanmis oturumun **odak suresi** (gecen sure eksi kesintiler).
    ///
    /// Odak suresi asla negatif olmaz; kesinti gecen sureyi asarsa 0 doner.
    pub fn odak_dakika(&self) -> Sonuc<i32> {
        let gecen = self.gecen_dakika_toplam()?;
        let odak = gecen - i64::from(self.toplam_kesinti());
        Ok(if odak < 0 { 0 } else { odak as i32 })
    }

    /// Tek satirlik ozet: `#1 odak #3 [tamamlandi] 22 dk, 1 kesinti / 3 dk (kapali)`.
    pub fn ozet(&self) -> Sonuc<String> {
        let gorev = match self.gorev_id {
            Some(id) => format!(" #{id}"),
            None => String::new(),
        };
        let kapanis = match self.bitis {
            Some(_) => "kapali",
            None => "acik",
        };
        let odak = if self.kapali() {
            format!("{} dk", self.odak_dakika()?)
        } else {
            format!("{} dk planli", self.planli_dakika)
        };
        let kesinti_kismi = if self.kesintiler.is_empty() {
            String::new()
        } else {
            format!(
                ", {} kesinti / {} dk",
                self.kesintiler.len(),
                self.toplam_kesinti()
            )
        };
        Ok(format!(
            "#{} {}{gorev} [{}] {odak}{kesinti_kismi} ({kapanis})",
            self.sira,
            self.tur.ad(),
            self.durum.ad()
        ))
    }
}

/// Oturum gunlugu: tum oturumlar ve acik oturum denetimi.
#[derive(Debug, Clone, Default)]
pub struct OturumGunlugu {
    oturumlar: Vec<Oturum>,
}

impl OturumGunlugu {
    /// Bos gunluk olusturur.
    pub fn yeni() -> Self {
        Self {
            oturumlar: Vec::new(),
        }
    }

    /// Gunluktaki tum oturumlar (sira numarasina gore sirali).
    pub fn oturumlar(&self) -> &[Oturum] {
        &self.oturumlar
    }

    /// Gunluktaki oturum sayisi.
    pub fn uzunluk(&self) -> usize {
        self.oturumlar.len()
    }

    /// Gunluk bos mu.
    pub fn bos_mu(&self) -> bool {
        self.oturumlar.is_empty()
    }

    /// Halen acik olan oturum (varsa).
    pub fn acik_oturum(&self) -> Option<&Oturum> {
        self.oturumlar.iter().find(|o| !o.kapali())
    }

    /// Verilen siradaki oturumu dondurur.
    pub fn getir(&self, sira: u32) -> Sonuc<&Oturum> {
        self.oturumlar
            .iter()
            .find(|o| o.sira == sira)
            .ok_or(Hata::BilinmeyenGorev { id: sira })
    }

    /// Verilen siradaki oturumu degistirilebilir sekilde dondurur.
    pub fn getir_mut(&mut self, sira: u32) -> Sonuc<&mut Oturum> {
        self.oturumlar
            .iter_mut()
            .find(|o| o.sira == sira)
            .ok_or(Hata::BilinmeyenGorev { id: sira })
    }

    /// Gunluge hazirlanmis bir oturumu ekler (sira numarasi korunur).
    pub fn ekle(&mut self, oturum: Oturum) {
        self.oturumlar.push(oturum);
        self.oturumlar.sort_by_key(|o| o.sira);
    }

    /// JSONL dosyasindan okunan kayitlardan gunluk kurar.
    ///
    /// Oturum gunlugu **eklenebilir** bir dosyadir ve kapanan bir oturum yeni bir
    /// satir olarak yazilir. Ayni sira numarasi birden fazla kez geciyorsa
    /// **son kayit gecerlidir**; aksi halde dosyada hep acik kalan eski kayit
    /// gorulur ve oturum bitmis sayilmazdi.
    pub fn yukle(ham: Vec<Oturum>) -> Self {
        let mut sirali = ham;
        sirali.sort_by_key(|o| o.sira);
        let mut gunluk = Self::yeni();
        for oturum in sirali {
            match gunluk.oturumlar.last() {
                Some(onceki) if onceki.sira == oturum.sira => {
                    let son = gunluk.oturumlar.len() - 1;
                    gunluk.oturumlar[son] = oturum;
                }
                _ => gunluk.oturumlar.push(oturum),
            }
        }
        gunluk
    }

    /// Yeni oturum baslatir ve sirasini dondurur.
    ///
    /// Gunlukte zaten acik bir oturum varsa baslatilir: **ust uste pomodoro
    /// calismaz**. Kapanmis oturumlar arasindaki zaman araligi cakismasi
    /// (elle duzenlenmis gunluk) `cakisan_ciftler` ile ayrica denetlenir.
    pub fn baslat(
        &mut self,
        gorev_id: Option<u32>,
        tur: OturumTuru,
        planli_dakika: i32,
        baslangic: i64,
    ) -> Sonuc<u32> {
        if let Some(acik) = self.acik_oturum() {
            return Err(Hata::OturumCakisiyor {
                acik_sira: acik.sira,
            });
        }
        let sira = self.oturumlar.iter().map(|o| o.sira).max().unwrap_or(0) + 1;
        let oturum = Oturum::yeni(sira, gorev_id, tur, planli_dakika, baslangic)?;
        self.oturumlar.push(oturum);
        Ok(sira)
    }

    /// Siradaki acik oturumu kapatir.
    pub fn acik_oturumu_bitir(&mut self, bitis: i64) -> Sonuc<u32> {
        let sira = self
            .acik_oturum()
            .map(|o| o.sira)
            .ok_or(Hata::AcikOturumYok)?;
        self.getir_mut(sira)?.bitir(bitis)?;
        Ok(sira)
    }

    /// Iki oturumun zaman araliklari ustelmis ise ciftlerini dondurur.
    ///
    /// Normal kullanimda bu liste bosdur; elle duzenlenmis veya bozuk gunlukleri
    /// yakalamak icin vardir. Ayni odurum kendsiyle cakismaz.
    pub fn cakisan_ciftler(&self) -> Vec<(u32, u32)> {
        let mut ciftler = Vec::new();
        for (i, a) in self.oturumlar.iter().enumerate() {
            for b in self.oturumlar.iter().skip(i + 1) {
                if oturumlar_cakisiyor(a, b) {
                    ciftler.push((a.sira, b.sira));
                }
            }
        }
        ciftler
    }

    /// `[bas, son)` araligina dusen oturumlar (son haric).
    pub fn araliktakiler(&self, bas: i64, son: i64) -> Vec<&Oturum> {
        self.oturumlar
            .iter()
            .filter(|o| o.baslangic >= bas && o.baslangic < son)
            .collect()
    }

    /// Bir gun icindeki oturumlar.
    ///
    /// Gece yarisi asan bir oturum, **basladigi** gunun sayilir; boylece gece
    /// yarisi gecisinde gun toplamlari kaymaz.
    pub fn gun_ici(&self, gun_baslangic: i64, gun_saniye: i64) -> Vec<&Oturum> {
        self.araliktakiler(gun_baslangic, gun_baslangic + gun_saniye)
    }

    /// Araliktaki **odak** oturumlarinin toplam odak suresi (dakika).
    pub fn toplam_odak_dakika(&self, bas: i64, son: i64) -> Sonuc<i32> {
        let mut toplam = 0;
        for oturum in self.araliktakiler(bas, son) {
            if oturum.tur.odak_mi() && oturum.kapali() {
                toplam += oturum.odak_dakika()?;
            }
        }
        Ok(toplam)
    }

    /// Araliktaki tum oturumlarin toplam kesinti suresi (dakika).
    pub fn toplam_kesinti_dakika(&self, bas: i64, son: i64) -> i32 {
        self.araliktakiler(bas, son)
            .iter()
            .map(|o| o.toplam_kesinti())
            .sum()
    }

    /// Kesinti kaynaklarinin toplam suresi (dakika), alfabetik sirali.
    pub fn kaynak_dagilimi(&self, bas: i64, son: i64) -> Vec<(String, i32)> {
        let mut toplamlar: Vec<(String, i32)> = Vec::new();
        for oturum in self.araliktakiler(bas, son) {
            for kesinti in &oturum.kesintiler {
                match toplamlar.iter_mut().find(|(k, _)| *k == kesinti.kaynak) {
                    Some((_, dakika)) => *dakika += kesinti.dakika,
                    None => toplamlar.push((kesinti.kaynak.clone(), kesinti.dakika)),
                }
            }
        }
        toplamlar.sort();
        toplamlar
    }
}

/// Iki oturumun zaman araligi ustelmis mi.
fn oturumlar_cakisiyor(a: &Oturum, b: &Oturum) -> bool {
    let a_bas = a.baslangic;
    let a_bit = a.bitis.unwrap_or(i64::MAX);
    let b_bas = b.baslangic;
    let b_bit = b.bitis.unwrap_or(i64::MAX);
    a_bas < b_bit && b_bas < a_bit
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    const DAKIKA: i64 = 60;

    fn odak(sira: u32, gorev: u32, bas: i64, planli: i32) -> Oturum {
        Oturum::yeni(sira, Some(gorev), OturumTuru::Odak, planli, bas).unwrap()
    }

    #[test]
    fn sifir_planli_sure_reddedilir() {
        let sonuc = Oturum::yeni(1, Some(1), OturumTuru::Odak, 0, 0);
        assert!(matches!(sonuc, Err(Hata::SifirSure)));
    }

    #[test]
    fn negatif_planli_sure_reddedilir() {
        let sonuc = Oturum::yeni(1, Some(1), OturumTuru::Odak, -25, 0);
        assert!(matches!(sonuc, Err(Hata::NegatifSure { deger: -25 })));
    }

    #[test]
    fn tur_ayrimasi_ve_adlari() {
        assert_eq!(OturumTuru::ayir("odak").unwrap(), OturumTuru::Odak);
        assert_eq!(OturumTuru::ayir("kisa").unwrap(), OturumTuru::KisaMola);
        assert_eq!(OturumTuru::ayir("uzun").unwrap(), OturumTuru::UzunMola);
        assert!(OturumTuru::ayir("bilinmeyen").is_err());
        assert!(OturumTuru::Odak.odak_mi());
        assert!(!OturumTuru::UzunMola.odak_mi());
    }

    #[test]
    fn ustelte_oturum_baslatilamaz() {
        let mut g = OturumGunlugu::yeni();
        g.baslat(Some(1), OturumTuru::Odak, 25, 1_000).unwrap();
        let sonuc = g.baslat(Some(2), OturumTuru::Odak, 25, 1_100);
        assert!(matches!(sonuc, Err(Hata::OturumCakisiyor { acik_sira: 1 })));
        assert_eq!(g.uzunluk(), 1, "ikinci oturum eklenmemeli");
        // ilk oturum kapaninca yenisi baslatilabilir
        g.acik_oturumu_bitir(1_000 + 25 * DAKIKA).unwrap();
        assert!(g.acik_oturum().is_none());
        g.baslat(Some(2), OturumTuru::Odak, 25, 2_000).unwrap();
        assert_eq!(g.uzunluk(), 2);
    }

    #[test]
    fn acilis_kapanis_durumu_yalin() {
        let mut o = odak(1, 1, 1_000, 25);
        assert!(!o.kapali());
        o.bitir(1_000 + 25 * DAKIKA).unwrap();
        assert!(o.kapali());
        assert_eq!(o.durum, OturumDurumu::Tamamlandi);
        assert_eq!(o.gecen_dakika_toplam().unwrap(), 25);
    }

    #[test]
    fn yarim_kalan_oturum_birakildi_isaretlenir() {
        let mut o = odak(1, 1, 1_000, 25);
        o.bitir(1_000 + 10 * DAKIKA).unwrap();
        assert_eq!(o.durum, OturumDurumu::Birakildi);
        assert_eq!(o.odak_dakika().unwrap(), 10);
        assert!(o.ozet().unwrap().contains("birakildi"));
    }

    #[test]
    fn kapanmis_oturum_tekrar_kapatilamaz() {
        let mut o = odak(1, 1, 1_000, 25);
        o.bitir(1_500).unwrap();
        let sonuc = o.bitir(2_000);
        assert!(matches!(sonuc, Err(Hata::OturumKapatilmis { sira: 1 })));
    }

    #[test]
    fn bitis_baslangictan_once_reddedilir() {
        let mut o = odak(1, 1, 1_000, 25);
        let sonuc = o.bitir(999);
        assert!(matches!(
            sonuc,
            Err(Hata::BitisBaslangictanOnce { sira: 1 })
        ));
    }

    #[test]
    fn kesinti_toplami_ve_odak_suresi_ayrilir() {
        let mut o = odak(1, 1, 1_000, 25);
        o.kesinti_ekle("toplanti", 5, 25).unwrap();
        o.kesinti_ekle("mesaj", 3, 25).unwrap();
        assert_eq!(o.toplam_kesinti(), 8);
        o.bitir(1_000 + 25 * DAKIKA).unwrap();
        assert_eq!(o.odak_dakika().unwrap(), 17);
        let ozet = o.ozet().unwrap();
        assert!(ozet.contains("2 kesinti"), "ozet: {ozet}");
        assert!(ozet.contains("8 dk"), "ozet: {ozet}");
    }

    #[test]
    fn kesinti_gecen_sureyi_asan_reddedilir() {
        let mut o = odak(1, 1, 1_000, 25);
        assert!(o.kesinti_ekle("toplanti", 5, 25).is_ok());
        let sonuc = o.kesinti_ekle("toplanti", 21, 25);
        assert!(matches!(
            sonuc,
            Err(Hata::KesintiAsim {
                toplam: 26,
                gecen: 25
            })
        ));
        assert_eq!(o.toplam_kesinti(), 5, "reddedilen kesinti eklenmemeli");
    }

    #[test]
    fn sifir_ve_negatif_kesinti_reddedilir() {
        let mut o = odak(1, 1, 1_000, 25);
        assert!(matches!(o.kesinti_ekle("x", 0, 25), Err(Hata::SifirSure)));
        assert!(matches!(
            o.kesinti_ekle("x", -1, 25),
            Err(Hata::NegatifSure { deger: -1 })
        ));
        assert!(
            o.kesinti_ekle("   ", 3, 25).is_err(),
            "bos kaynak reddedilir"
        );
    }

    #[test]
    fn gece_yari_asil_oturum_basladigi_gune_yazilir() {
        // 2026-09-29 23:50 yerel -> 00:10 (gece yarisi gecisi)
        let gun_bas = 1_000_000 * 86_400;
        let bas = gun_bas + 23 * 3_600 + 50 * 60;
        let bit = gun_bas + 86_400 + 10 * 60;
        let mut g = OturumGunlugu::yeni();
        let mut o = Oturum::yeni(1, Some(1), OturumTuru::Odak, 25, bas).unwrap();
        o.kesinti_ekle("toplanti", 5, 20).unwrap();
        o.bitir(bit).unwrap();
        g.ekle(o);
        assert_eq!(g.gun_ici(gun_bas, 86_400).len(), 1, "basladigi gun sayilir");
        assert_eq!(
            g.gun_ici(gun_bas + 86_400, 86_400).len(),
            0,
            "bittigi gun sayilmaz"
        );
        assert_eq!(
            g.toplam_odak_dakika(gun_bas, gun_bas + 2 * 86_400).unwrap(),
            15
        );
    }

    #[test]
    fn acilis_oturumu_toplamlara_girmez() {
        let gun_bas = 5 * 86_400;
        let mut g = OturumGunlugu::yeni();
        g.baslat(Some(1), OturumTuru::Odak, 25, gun_bas).unwrap();
        assert_eq!(g.toplam_odak_dakika(gun_bas, gun_bas + 86_400).unwrap(), 0);
    }

    #[test]
    fn mola_oturumu_odak_toplamina_girmez() {
        let gun_bas = 5 * 86_400;
        let mut g = OturumGunlugu::yeni();
        let sira = g.baslat(Some(1), OturumTuru::Odak, 25, gun_bas).unwrap();
        g.getir_mut(sira).unwrap().bitir(gun_bas + 25 * 60).unwrap();
        let sira = g
            .baslat(None, OturumTuru::KisaMola, 5, gun_bas + 25 * 60)
            .unwrap();
        g.getir_mut(sira).unwrap().bitir(gun_bas + 30 * 60).unwrap();
        assert_eq!(g.toplam_odak_dakika(gun_bas, gun_bas + 86_400).unwrap(), 25);
    }

    #[test]
    fn cakisik_kayit_ciftleri_tespit_edilir() {
        let mut g = OturumGunlugu::yeni();
        // elle bozulmus gunluk: ust uste iki oturum
        let mut a = odak(1, 1, 1_000, 25);
        a.bitir(3_000).unwrap();
        let mut b = odak(2, 2, 2_000, 25);
        b.bitir(4_000).unwrap();
        g.ekle(a);
        g.ekle(b);
        assert_eq!(g.cakisan_ciftler(), vec![(1, 2)]);
    }

    #[test]
    fn ardisik_oturumlar_cakismaz() {
        let mut g = OturumGunlugu::yeni();
        let sira = g.baslat(Some(1), OturumTuru::Odak, 25, 1_000).unwrap();
        g.getir_mut(sira).unwrap().bitir(2_500).unwrap();
        g.baslat(None, OturumTuru::KisaMola, 5, 2_500).unwrap();
        g.acik_oturumu_bitir(2_800).unwrap();
        assert!(g.cakisan_ciftler().is_empty());
    }

    #[test]
    fn kaynak_dagilimi_hesaplanir() {
        let gun_bas = 7 * 86_400;
        let mut g = OturumGunlugu::yeni();
        for (kaynak, dakika, bas_dk) in
            [("toplanti", 10, 0), ("mesaj", 5, 100), ("toplanti", 7, 200)]
        {
            let bas = gun_bas + bas_dk * 60;
            let sira = g.baslat(Some(1), OturumTuru::Odak, 25, bas).unwrap();
            let oturum = g.getir_mut(sira).unwrap();
            oturum.kesinti_ekle(kaynak, dakika, 25).unwrap();
            oturum.bitir(bas + 25 * 60).unwrap();
        }
        assert_eq!(
            g.kaynak_dagilimi(gun_bas, gun_bas + 86_400),
            vec![("mesaj".to_string(), 5), ("toplanti".to_string(), 17)]
        );
    }

    #[test]
    fn acik_oturum_bitirilemezse_hata_verir() {
        let mut g = OturumGunlugu::yeni();
        let sonuc = g.acik_oturumu_bitir(1_000);
        assert!(matches!(sonuc, Err(Hata::AcikOturumYok)));
    }

    #[test]
    fn ayni_sira_iki_kez_varsa_son_kayit_gecerlidir() {
        let mut g = OturumGunlugu::yeni();
        g.baslat(Some(1), OturumTuru::Odak, 25, 1_000).unwrap();
        let mut kapali = g.getir(1).unwrap().clone();
        kapali.bitir(2_500).unwrap();
        // JSONL'e acik kayit ve kapali kayit art arda yazilmis olabilir.
        let gecmis = vec![
            Oturum::yeni(1, Some(1), OturumTuru::Odak, 25, 1_000).unwrap(),
            kapali,
        ];
        let yuklenen = OturumGunlugu::yukle(gecmis);
        assert_eq!(yuklenen.uzunluk(), 1, "ayni sira tek oturum sayilir");
        assert!(yuklenen.acik_oturum().is_none());
        assert_eq!(yuklenen.getir(1).unwrap().odak_dakika().unwrap(), 25);
    }
}
