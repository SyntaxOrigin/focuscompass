//! Gorev deposu: kimlik, baslik, tahmini sure, oncelik, durum, etiket ve bagimlilik.
//!
//! Bagimlilik grafigi **kendi kendine** kurulur ve DFS (renkli, uc uca) ile
//! denetlenir; dongu olusmaya calisan bir bagimlilik kaydedilmez. Bu, fikir
//! raporunun R.1/R.2 riskine karsi temel garanti: veri girisi hatasi araci
//! kilitlemez, yalnizca reddeder.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::hata::{Hata, Sonuc};

/// Bir gorevin yasam durumu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Durum {
    /// Henuz baslanmadi.
    Bekliyor,
    /// Su anda calisiliyor.
    Calisiyor,
    /// Basariyla bitti.
    Tamamlandi,
    /// Sonraya birakildi.
    Ertelendi,
}

impl Durum {
    /// Durumun veri dosyasindaki ASCII adi.
    pub fn ad(self) -> &'static str {
        match self {
            Self::Bekliyor => "bekliyor",
            Self::Calisiyor => "calisiyor",
            Self::Tamamlandi => "tamamlandi",
            Self::Ertelendi => "ertelendi",
        }
    }

    /// ASCII adindan durum cozer.
    pub fn ayir(metin: &str) -> Sonuc<Self> {
        match metin {
            "bekliyor" => Ok(Self::Bekliyor),
            "calisiyor" => Ok(Self::Calisiyor),
            "tamamlandi" => Ok(Self::Tamamlandi),
            "ertelendi" => Ok(Self::Ertelendi),
            _ => Err(Hata::GecersizDurumAdi {
                metin: metin.to_string(),
            }),
        }
    }

    /// Durum makinesinin bu gecise izin verip vermedigi.
    ///
    /// Gecersiz olan iki gecis bilincli olarak yaslidir: `tamamlandi -> ertelendi`
    /// (bitmis bir isi ertelemek anlamsiz) ve `ertelendi -> tamamlandi`
    /// (bir isareti gecersiz kilmadan bitmis saymak anlamsiz; once bekleme durumu).
    pub fn gecis_gecerli(self, hedef: Durum) -> bool {
        matches!(
            (self, hedef),
            (Self::Bekliyor, Self::Calisiyor)
                | (Self::Bekliyor, Self::Tamamlandi)
                | (Self::Bekliyor, Self::Ertelendi)
                | (Self::Calisiyor, Self::Tamamlandi)
                | (Self::Calisiyor, Self::Bekliyor)
                | (Self::Calisiyor, Self::Ertelendi)
                | (Self::Tamamlandi, Self::Bekliyor)
                | (Self::Tamamlandi, Self::Calisiyor)
                | (Self::Ertelendi, Self::Bekliyor)
                | (Self::Ertelendi, Self::Calisiyor)
        )
    }

    /// Gorevin hala acik (yani bitmemis) olup olmadigi.
    pub fn acik(self) -> bool {
        matches!(self, Self::Bekliyor | Self::Calisiyor)
    }
}

/// Gorevin oncelik sirasi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Oncelik {
    /// En dusuk oncelik.
    Dusuk,
    /// Varsayilan oncelik.
    Normal,
    /// En yuksek oncelik.
    Yuksek,
}

impl Oncelik {
    /// ASCII ad.
    pub fn ad(self) -> &'static str {
        match self {
            Self::Dusuk => "dusuk",
            Self::Normal => "normal",
            Self::Yuksek => "yuksek",
        }
    }

    /// ASCII adindan oncelik cozer.
    pub fn ayir(metin: &str) -> Sonuc<Self> {
        match metin {
            "dusuk" => Ok(Self::Dusuk),
            "normal" => Ok(Self::Normal),
            "yuksek" => Ok(Self::Yuksek),
            _ => Err(Hata::GecersizOncelik {
                metin: metin.to_string(),
            }),
        }
    }
}

/// Tek bir gorev kaydi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gorev {
    /// Depo icinde benzersiz kimlik; 1'den baslar.
    pub id: u32,
    /// Kisaltilmamis gorev basligi.
    pub baslik: String,
    /// Tahmini sure (dakika). Negatif olamaz.
    pub tahmini_dakika: i32,
    /// Oncelik sirasi.
    pub oncelik: Oncelik,
    /// Yasm durumu.
    pub durum: Durum,
    /// Normalize edilmis etiketler (kucuk harf, sirali, tekrarsiz).
    pub etiketler: Vec<String>,
    /// Bu gorevin bitmeden once bitmesi gereken gorev kimlikleri.
    pub bagimliliklar: Vec<u32>,
    /// Olusturulma zamani (UTC epoch saniye).
    pub olusturma: i64,
    /// Son durum degisikligi (UTC epoch saniye).
    pub guncelleme: i64,
}

impl Gorev {
    /// Yeni bir gorev olusturur ve girdiyi dogrular.
    ///
    /// Baslik kirpilir; bos kalirsa `Hata::BosBaslik`. Tahmini sure negatifse
    /// `Hata::NegatifSure`. Sifir tahmin kabul edilir: "suresi bilinmiyor"
    /// anlamina gelir ve kapasite hesabina katilmaz.
    pub fn yeni(
        id: u32,
        baslik: &str,
        tahmini_dakika: i32,
        oncelik: Oncelik,
        etiketler: &[String],
        simdi: i64,
    ) -> Sonuc<Self> {
        let baslik = baslik.trim().to_string();
        if baslik.is_empty() {
            return Err(Hata::BosBaslik);
        }
        if tahmini_dakika < 0 {
            return Err(Hata::NegatifSure {
                deger: tahmini_dakika,
            });
        }
        Ok(Self {
            id,
            baslik,
            tahmini_dakika,
            oncelik,
            durum: Durum::Bekliyor,
            etiketler: etiket_normalize(etiketler)?,
            bagimliliklar: Vec::new(),
            olusturma: simdi,
            guncelleme: simdi,
        })
    }
}

/// Etiket listesini normalizes eder.
///
/// Kurallar: bastaki `#` atilir, kenar bosluklari kirpilir, Turkce buyuk
/// harfler dogru kucultulur (`I` -> `ı`, `İ` -> `i`), ic bosluklar `-` olur,
/// tekrarlar atilir ve sonuc alfabetik siralanir. Bos veya yasak karakter
/// iceren etiket `Hata::GecersizEtiket` ile reddedilir.
pub fn etiket_normalize(etiketler: &[String]) -> Sonuc<Vec<String>> {
    let mut sonuc: Vec<String> = Vec::new();
    for ham in etiketler {
        let temizlenmis = etiket_kucult(ham.trim());
        let gecerli = !temizlenmis.is_empty()
            && temizlenmis
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_');
        if !gecerli {
            return Err(Hata::GecersizEtiket {
                etiket: ham.clone(),
            });
        }
        if !sonuc.contains(&temizlenmis) {
            sonuc.push(temizlenmis);
        }
    }
    sonuc.sort();
    Ok(sonuc)
}

/// Turkce kurallarina uygun kucuk harfe cevirir ve bosluklari `-` yapar.
fn etiket_kucult(metin: &str) -> String {
    let kurulmus = metin.trim_start_matches('#').trim();
    let yerine_koyulmus: String = kurulmus
        .chars()
        .map(|c| match c {
            'I' => '\u{131}', // I -> ı
            '\u{130}' => 'i', // İ -> i
            ' ' | '\t' => '-',
            diger => diger,
        })
        .collect();
    yerine_koyulmus.to_lowercase()
}

/// `add` komutundan gelen, dogrulanmamis gorev girdisi.
#[derive(Debug, Clone)]
pub struct GorevGirdi {
    /// Gorev basligi.
    pub baslik: String,
    /// Tahmini sure (dakika).
    pub tahmini_dakika: i32,
    /// Oncelik.
    pub oncelik: Oncelik,
    /// Ham etiketler (normalize edilir).
    pub etiketler: Vec<String>,
    /// Bagimlilik kimlikleri.
    pub bagimliliklar: Vec<u32>,
}

impl Default for GorevGirdi {
    fn default() -> Self {
        Self {
            baslik: String::new(),
            tahmini_dakika: 25,
            oncelik: Oncelik::Normal,
            etiketler: Vec::new(),
            bagimliliklar: Vec::new(),
        }
    }
}

/// Gorev deposu: kimlik -> gorev eslemesi ve bagimlilik grafigi.
#[derive(Debug, Clone, Default)]
pub struct GorevDeposu {
    gorevler: Vec<Gorev>,
    sonraki_id: u32,
}

impl GorevDeposu {
    /// Bos depo olusturur.
    pub fn yeni() -> Self {
        Self {
            gorevler: Vec::new(),
            sonraki_id: 1,
        }
    }

    /// Verilen gorev listesinden depo kurar; kimlikler tekrarlanamaz.
    ///
    /// Kimlik verilmemis (0) gorevlere sirayla kimlik atar.
    pub fn gorevlerden(gorevler: Vec<Gorev>) -> Sonuc<Self> {
        let mut depo = Self::yeni();
        for mut gorev in gorevler {
            if gorev.id == 0 {
                gorev.id = depo.sonraki_id;
            }
            if depo.gorevler.iter().any(|g| g.id == gorev.id) {
                return Err(Hata::TekrarliKimlik { id: gorev.id });
            }
            if gorev.tahmini_dakika < 0 {
                return Err(Hata::NegatifSure {
                    deger: gorev.tahmini_dakika,
                });
            }
            if gorev.id >= depo.sonraki_id {
                depo.sonraki_id = gorev.id + 1;
            }
            depo.gorevler.push(gorev);
        }
        depo.gorevler.sort_by_key(|g| g.id);
        Ok(depo)
    }

    /// Depodaki tum gorevler (kimlige gore sirali).
    pub fn tum(&self) -> &[Gorev] {
        &self.gorevler
    }

    /// Depodaki gorev sayisi.
    pub fn uzunluk(&self) -> usize {
        self.gorevler.len()
    }

    /// Depo bos mu.
    pub fn bos_mu(&self) -> bool {
        self.gorevler.is_empty()
    }

    /// Kimlige gore gorev getirir.
    pub fn getir(&self, id: u32) -> Sonuc<&Gorev> {
        self.gorevler
            .iter()
            .find(|g| g.id == id)
            .ok_or(Hata::BilinmeyenGorev { id })
    }

    /// Kimlige gore gorev verir.
    pub fn getir_mut(&mut self, id: u32) -> Sonuc<&mut Gorev> {
        self.gorevler
            .iter_mut()
            .find(|g| g.id == id)
            .ok_or(Hata::BilinmeyenGorev { id })
    }

    /// Yeni gorev ekler ve kimligini dondurur.
    ///
    /// Bagimliliklar sirasiyla eklenir; biri reddedilirse **hiçbir gorev
    /// eklenmez** (kismi yazim olusmaz) ve kimlik numarasi geri sarilir.
    pub fn ekle(&mut self, girdi: GorevGirdi, simdi: i64) -> Sonuc<u32> {
        let id = self.sonraki_id;
        let gorev = Gorev::yeni(
            id,
            &girdi.baslik,
            girdi.tahmini_dakika,
            girdi.oncelik,
            &girdi.etiketler,
            simdi,
        )?;
        // Bagimlilik ekleme yeni gorevi depoda arar; bu yuzden gorev once yazilir.
        self.gorevler.push(gorev);
        for bagimlilik in &girdi.bagimliliklar {
            if let Err(hata) = self.bagimlilik_ekle_ic(id, *bagimlilik) {
                self.gorevler.retain(|g| g.id != id);
                return Err(hata);
            }
        }
        self.sonraki_id += 1;
        Ok(id)
    }

    /// Gorevin durumunu degistirir; gecis makinesi gecersizse reddeder.
    pub fn durum_degistir(&mut self, id: u32, hedef: Durum, simdi: i64) -> Sonuc<()> {
        let gorev = self.getir(id)?;
        let kaynak = gorev.durum;
        if kaynak == hedef {
            return Ok(());
        }
        if !kaynak.gecis_gecerli(hedef) {
            return Err(Hata::GecersizGecis { kaynak, hedef });
        }
        let gorev = self.getir_mut(id)?;
        gorev.durum = hedef;
        gorev.guncelleme = simdi;
        Ok(())
    }

    /// Goreve bagimlilik ekler.
    ///
    /// Kendine baglanma, bilinmeyen kimlik ve dongu reddedilir; reddedilen
    /// bagimlilik **kaydedilmez**.
    pub fn bagimlilik_ekle(&mut self, id: u32, bagimlilik: u32) -> Sonuc<()> {
        self.bagimlilik_ekle_ic(id, bagimlilik)
    }

    fn bagimlilik_ekle_ic(&mut self, id: u32, bagimlilik: u32) -> Sonuc<()> {
        if id == bagimlilik {
            return Err(Hata::KendineBagimlilik { id });
        }
        let hedef_var = self.gorevler.iter().any(|g| g.id == bagimlilik);
        if !hedef_var {
            return Err(Hata::BilinmeyenBagimlilik { id, bagimlilik });
        }
        if self.getir(id)?.bagimliliklar.contains(&bagimlilik) {
            return Err(Hata::TekrarliBagimlilik { id, bagimlilik });
        }
        self.getir_mut(id)?.bagimliliklar.push(bagimlilik);
        self.getir_mut(id)?.bagimliliklar.sort_unstable();
        match self.dongu_ara() {
            Some(yol) => {
                let gorev = self.getir_mut(id)?;
                gorev.bagimliliklar.retain(|b| *b != bagimlilik);
                Err(Hata::BagimlilikDongusu { yol })
            }
            None => Ok(()),
        }
    }

    /// Gorevden bagimliligi kaldirir.
    pub fn bagimlilik_kaldir(&mut self, id: u32, bagimlilik: u32) -> Sonuc<()> {
        let gorev = self.getir_mut(id)?;
        let onceki = gorev.bagimliliklar.len();
        gorev.bagimliliklar.retain(|b| *b != bagimlilik);
        if gorev.bagimliliklar.len() == onceki {
            return Err(Hata::BilinmeyenBagimlilik { id, bagimlilik });
        }
        Ok(())
    }

    /// Gorevi tamamlanmamis bagimliliklariyla birlikte dondurur.
    pub fn tamamlanmamis_bagimliliklar(&self, id: u32) -> Sonuc<Vec<(u32, Durum)>> {
        let gorev = self.getir(id)?;
        let mut eksik = Vec::new();
        for bagimlilik in &gorev.bagimliliklar {
            let durum = self.getir(*bagimlilik)?.durum;
            if durum != Durum::Tamamlandi {
                eksik.push((*bagimlilik, durum));
            }
        }
        Ok(eksik)
    }

    /// Gorevin baslatilip baslatilamayacagini denetler.
    ///
    /// Ertelemis bir bagimlilik da engeldir: ertelenen bir is, "bitmis" sayilmaz.
    pub fn baslatilabilir(&self, id: u32) -> Sonuc<()> {
        match self.tamamlanmamis_bagimliliklar(id)?.first() {
            None => Ok(()),
            Some(&(engelleyen, durum)) => Err(Hata::BaslatilamazSebep {
                id,
                engelleyen,
                durum,
            }),
        }
    }

    /// Gorevin tam gecisli bagimlilik agacini dondurur (gorevden en derin bagimliga).
    ///
    /// Depoda dongu olmadigi varsayilir; `denetle` ile teyit edilmelidir.
    pub fn bagimlilik_agaci(&self, id: u32) -> Sonuc<Vec<Vec<u32>>> {
        self.getir(id)?;
        let mut katmanlar = Vec::new();
        let mut mevcut = vec![id];
        let mut ziyaret = vec![id];
        while let Some(&ust) = mevcut.first() {
            let gorev = self.getir(ust)?;
            let altlar: Vec<u32> = gorev
                .bagimliliklar
                .iter()
                .copied()
                .filter(|b| !ziyaret.contains(b))
                .collect();
            if altlar.is_empty() {
                break;
            }
            for alt in &altlar {
                self.getir(*alt)?;
                ziyaret.push(*alt);
            }
            katmanlar.push(altlar.clone());
            mevcut = altlar;
        }
        Ok(katmanlar)
    }

    /// Depodaki tum bagimliliklarin var oldugunu ve dongu icermedigini dogrular.
    ///
    /// Elle duzenlenmis dosyalar bu denetimden gecer; hatali alan reddedilmez,
    /// dosya reddedilir (rapor b03 / S6 "dosya reddedilmez, hatali alanlar
    /// isaretlenir" ilkesi bu MVP'de "satiri reddet" olarak uygulanir).
    pub fn denetle(&self) -> Sonuc<()> {
        for gorev in &self.gorevler {
            for bagimlilik in &gorev.bagimliliklar {
                if !self.gorevler.iter().any(|g| g.id == *bagimlilik) {
                    return Err(Hata::BilinmeyenBagimlilik {
                        id: gorev.id,
                        bagimlilik: *bagimlilik,
                    });
                }
            }
        }
        if let Some(yol) = self.dongu_ara() {
            return Err(Hata::BagimlilikDongusu { yol });
        }
        Ok(())
    }

    /// Bagimlilik grafiginde dongu varsa yolunu dondurur.
    ///
    /// Uc noktali DFS (beyaz / gri / siyah) kullanilir. Bulunan yol
    /// `#[1, 2, 1]` seklindedir.
    pub fn dongu_ara(&self) -> Option<Vec<u32>> {
        let mut renk: HashMap<u32, u8> = HashMap::new();
        for gorev in &self.gorevler {
            if renk.get(&gorev.id).copied().unwrap_or(0) == 0 {
                let mut yol = vec![gorev.id];
                if self.dfs(gorev.id, &mut renk, &mut yol) {
                    return Some(yol);
                }
            }
        }
        None
    }

    /// Ozyinelemeli DFS; `yol` uzerinde gri bir dugume donulurse dongu bulundu.
    ///
    /// `yol` su anki yigindir (kirpma islemi yalnizca dongu bulundugunda yapilir),
    /// boylece bulunan dongu `#[1, 2, 3, 1]` seklinde tum dugumleriyle gorunur.
    fn dfs(&self, id: u32, renk: &mut HashMap<u32, u8>, yol: &mut Vec<u32>) -> bool {
        renk.insert(id, 1); // gri: yolun uzerinde
        let bagimliliklar = self
            .gorevler
            .iter()
            .find(|g| g.id == id)
            .map(|g| g.bagimliliklar.clone())
            .unwrap_or_default();
        for bagimlilik in bagimliliklar {
            match renk.get(&bagimlilik).copied().unwrap_or(0) {
                1 => {
                    yol.push(bagimlilik);
                    return true;
                }
                0 => {
                    yol.push(bagimlilik);
                    if self.dfs(bagimlilik, renk, yol) {
                        return true;
                    }
                    yol.pop();
                }
                _ => {}
            }
        }
        renk.insert(id, 2); // siyah: tamamlandi
        false
    }

    /// Oncelik azalan, tahmini sure artan, kimlik artan siralamasiyla gorevler.
    pub fn sirali(&self) -> Vec<&Gorev> {
        let mut sirali: Vec<&Gorev> = self.gorevler.iter().collect();
        sirali.sort_by(|a, b| {
            b.oncelik
                .cmp(&a.oncelik)
                .then(a.tahmini_dakika.cmp(&b.tahmini_dakika))
                .then(a.id.cmp(&b.id))
        });
        sirali
    }

    /// Duruma gore filtreler.
    pub fn duruma_gore(&self, durum: Durum) -> Vec<&Gorev> {
        self.gorevler.iter().filter(|g| g.durum == durum).collect()
    }

    /// Etikete gore filtreler; etiket onceden normalize edilir.
    pub fn etikete_gore(&self, etiket: &str) -> Sonuc<Vec<&Gorev>> {
        let normallestirilmis = etiket_normalize(&[etiket.to_string()])?;
        let aranan = normallestirilmis
            .first()
            .cloned()
            .ok_or_else(|| Hata::GecersizEtiket {
                etiket: etiket.to_string(),
            })?;
        Ok(self
            .gorevler
            .iter()
            .filter(|g| g.etiketler.contains(&aranan))
            .collect())
    }

    /// Bir etiketin kac gorevde kullanildigini dondurur.
    pub fn etiket_sayisi(&self, etiket: &str) -> Sonuc<usize> {
        Ok(self.etikete_gore(etiket)?.len())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn depo() -> GorevDeposu {
        GorevDeposu::yeni()
    }

    fn ekle(depo: &mut GorevDeposu, baslik: &str, dakika: i32) -> u32 {
        depo.ekle(
            GorevGirdi {
                baslik: baslik.to_string(),
                tahmini_dakika: dakika,
                oncelik: Oncelik::Normal,
                etiketler: Vec::new(),
                bagimliliklar: Vec::new(),
            },
            1_000,
        )
        .unwrap()
    }

    #[test]
    fn bos_baslik_reddedilir() {
        let sonuc = Gorev::yeni(1, "   ", 30, Oncelik::Normal, &[], 0);
        assert!(matches!(sonuc, Err(Hata::BosBaslik)));
    }

    #[test]
    fn negatif_tahmini_sure_reddedilir() {
        let sonuc = Gorev::yeni(1, "Rapor", -5, Oncelik::Normal, &[], 0);
        assert!(matches!(sonuc, Err(Hata::NegatifSure { deger: -5 })));
    }

    #[test]
    fn sifir_tahmini_sure_kabul_edilir() {
        let gorev = Gorev::yeni(1, "Belirsiz", 0, Oncelik::Normal, &[], 0).unwrap();
        assert_eq!(gorev.tahmini_dakika, 0);
        assert_eq!(gorev.durum, Durum::Bekliyor);
    }

    #[test]
    fn etiket_normalizasyonu_kucult_sirala_tekrarla() {
        let ham = vec![
            "#Rapor".to_string(),
            "  Mürekkep  ".to_string(),
            "rapor".to_string(),
            "IŞ TAKİBİ".to_string(),
        ];
        let sonuc = etiket_normalize(&ham).unwrap();
        assert_eq!(sonuc, vec!["mürekkep", "rapor", "ış-takibi"]);
    }

    #[test]
    fn gecersiz_etiket_reddedilir() {
        let ham = vec!["bir/iki".to_string()];
        assert!(matches!(
            etiket_normalize(&ham),
            Err(Hata::GecersizEtiket { .. })
        ));
        let bos = vec!["##".to_string()];
        assert!(etiket_normalize(&bos).is_err());
    }

    #[test]
    fn durum_makinesi_gecerli_gecisleri_kabul_eder() {
        let gecisler = [
            (Durum::Bekliyor, Durum::Calisiyor),
            (Durum::Bekliyor, Durum::Tamamlandi),
            (Durum::Bekliyor, Durum::Ertelendi),
            (Durum::Calisiyor, Durum::Tamamlandi),
            (Durum::Calisiyor, Durum::Bekliyor),
            (Durum::Calisiyor, Durum::Ertelendi),
            (Durum::Tamamlandi, Durum::Bekliyor),
            (Durum::Tamamlandi, Durum::Calisiyor),
            (Durum::Ertelendi, Durum::Bekliyor),
            (Durum::Ertelendi, Durum::Calisiyor),
        ];
        for (kaynak, hedef) in gecisler {
            assert!(kaynak.gecis_gecerli(hedef), "{kaynak:?} -> {hedef:?}");
        }
    }

    #[test]
    fn durum_makinesi_gecersiz_gecisi_reddeder() {
        assert!(!Durum::Tamamlandi.gecis_gecerli(Durum::Ertelendi));
        assert!(!Durum::Ertelendi.gecis_gecerli(Durum::Tamamlandi));
        assert!(!Durum::Tamamlandi.gecis_gecerli(Durum::Tamamlandi));
    }

    #[test]
    fn depo_durum_gecisi_uygular() {
        let mut d = depo();
        let id = ekle(&mut d, "Yaz", 45);
        d.durum_degistir(id, Durum::Calisiyor, 2_000).unwrap();
        assert_eq!(d.getir(id).unwrap().durum, Durum::Calisiyor);
        assert_eq!(d.getir(id).unwrap().guncelleme, 2_000);
        d.durum_degistir(id, Durum::Tamamlandi, 3_000).unwrap();
        let sonuc = d.durum_degistir(id, Durum::Ertelendi, 4_000);
        assert!(matches!(sonuc, Err(Hata::GecersizGecis { .. })));
    }

    #[test]
    fn iki_dugumlu_dongu_tespit_edilir() {
        let mut d = depo();
        let a = ekle(&mut d, "A", 10);
        let b = ekle(&mut d, "B", 10);
        d.bagimlilik_ekle(a, b).unwrap();
        // A -> B kuruldu; simdi B -> A deneniyor: halka olusur ve reddedilir.
        match d.bagimlilik_ekle(b, a) {
            Err(Hata::BagimlilikDongusu { yol }) => {
                assert_eq!(yol.first().copied(), Some(a), "yol: {yol:?}");
                assert_eq!(yol.last().copied(), Some(a), "yol: {yol:?}");
                assert!(yol.contains(&b), "yol: {yol:?}");
            }
            diger => panic!("dongu hatasi bekleniyordu, gelen: {diger:?}"),
        }
        assert!(
            d.dongu_ara().is_none(),
            "reddedilen bagimlilik kaydedilmemeli"
        );
    }

    #[test]
    fn uc_dugumlu_dongu_tespit_edilir() {
        let mut d = depo();
        let a = ekle(&mut d, "A", 10);
        let b = ekle(&mut d, "B", 10);
        let c = ekle(&mut d, "C", 10);
        d.bagimlilik_ekle(a, b).unwrap();
        d.bagimlilik_ekle(b, c).unwrap();
        assert!(d.dongu_ara().is_none());
        // c -> a kapatirsa uc dugumlu halka olusur ve reddedilir
        match d.bagimlilik_ekle(c, a) {
            Err(Hata::BagimlilikDongusu { yol }) => {
                assert_eq!(yol.len(), 4, "yol: {yol:?}");
            }
            diger => panic!("uc dugumlu dongu bekleniyordu, gelen: {diger:?}"),
        }
        assert!(d.dongu_ara().is_none());
        // elle bozulmus dosyada dongu tespit edilir
        let mut bozuk = d.clone();
        bozuk.getir_mut(c).unwrap().bagimliliklar.push(a);
        let yol = bozuk.dongu_ara().expect("dongu bulunamadi");
        assert_eq!(yol.len(), 4, "yol: {yol:?}");
    }

    #[test]
    fn kendine_bagimli_gorev_reddedilir() {
        let mut d = depo();
        let a = ekle(&mut d, "A", 10);
        let sonuc = d.bagimlilik_ekle(a, a);
        assert!(matches!(sonuc, Err(Hata::KendineBagimlilik { id } ) if id == a));
    }

    #[test]
    fn zincir_bagimlilik_acar_ve_izgara_gosterir() {
        let mut d = depo();
        let a = ekle(&mut d, "A", 10);
        let b = ekle(&mut d, "B", 10);
        let c = ekle(&mut d, "C", 10);
        d.bagimlilik_ekle(a, b).unwrap();
        d.bagimlilik_ekle(b, c).unwrap();
        d.durum_degistir(c, Durum::Tamamlandi, 10).unwrap();
        d.durum_degistir(b, Durum::Tamamlandi, 10).unwrap();
        assert!(d.baslatilabilir(a).is_ok());
        let agac = d.bagimlilik_agaci(a).unwrap();
        assert_eq!(agac, vec![vec![b], vec![c]]);
    }

    #[test]
    fn baslatilamayan_gorev_engellenen_bagimliligi_soyler() {
        let mut d = depo();
        let a = ekle(&mut d, "A", 10);
        let b = ekle(&mut d, "B", 10);
        d.bagimlilik_ekle(a, b).unwrap();
        let sonuc = d.baslatilabilir(a);
        assert!(matches!(
            sonuc,
            Err(Hata::BaslatilamazSebep {
                engelleyen,
                durum: Durum::Bekliyor,
                ..
            }) if engelleyen == b
        ));
    }

    #[test]
    fn ertelenmis_bagimlilik_gorevi_baslatir_engeller() {
        let mut d = depo();
        let a = ekle(&mut d, "A", 10);
        let b = ekle(&mut d, "B", 10);
        d.bagimlilik_ekle(a, b).unwrap();
        d.durum_degistir(b, Durum::Ertelendi, 10).unwrap();
        assert!(
            d.baslatilabilir(a).is_err(),
            "ertelenen bagimlilik engeldir"
        );
        // Erteleme geri alinca gorev "bekliyor" olur; bu **bitmis** sayilmaz.
        d.durum_degistir(b, Durum::Bekliyor, 20).unwrap();
        assert!(
            d.baslatilabilir(a).is_err(),
            "bekleyen bagimlilik da engeldir"
        );
        // Yalnizca "tamamlandi" bagimligi kaldirir.
        d.durum_degistir(b, Durum::Tamamlandi, 30).unwrap();
        assert!(d.baslatilabilir(a).is_ok());
    }

    #[test]
    fn tamamlanan_bagimlilik_engel_olmaz() {
        let mut d = depo();
        let a = ekle(&mut d, "A", 10);
        let b = ekle(&mut d, "B", 10);
        d.bagimlilik_ekle(a, b).unwrap();
        d.durum_degistir(b, Durum::Tamamlandi, 10).unwrap();
        assert!(d.baslatilabilir(a).is_ok());
        assert!(d.tamamlanmamis_bagimliliklar(a).unwrap().is_empty());
    }

    #[test]
    fn bilinmeyen_bagimlilik_reddedilir() {
        let mut d = depo();
        let a = ekle(&mut d, "A", 10);
        assert!(matches!(
            d.bagimlilik_ekle(a, 99),
            Err(Hata::BilinmeyenBagimlilik { .. })
        ));
        assert!(matches!(d.getir(99), Err(Hata::BilinmeyenGorev { id: 99 })));
    }

    #[test]
    fn tekrar_bagimlilik_reddedilir() {
        let mut d = depo();
        let a = ekle(&mut d, "A", 10);
        let b = ekle(&mut d, "B", 10);
        d.bagimlilik_ekle(a, b).unwrap();
        assert!(matches!(
            d.bagimlilik_ekle(a, b),
            Err(Hata::TekrarliBagimlilik { .. })
        ));
        d.bagimlilik_kaldir(a, b).unwrap();
        assert!(d.bagimlilik_ekle(a, b).is_ok());
    }

    #[test]
    fn depo_denetleme_asiili_bagimliligi_yakalar() {
        let mut d = depo();
        let a = ekle(&mut d, "A", 10);
        d.getir_mut(a).unwrap().bagimliliklar.push(77);
        assert!(matches!(
            d.denetle(),
            Err(Hata::BilinmeyenBagimlilik { .. })
        ));
    }

    #[test]
    fn ekleme_sirasinda_hatali_bagimlilik_geri_alinir() {
        let mut d = depo();
        let a = ekle(&mut d, "A", 10);
        let girdi = GorevGirdi {
            baslik: "B".to_string(),
            tahmini_dakika: 10,
            oncelik: Oncelik::Normal,
            etiketler: Vec::new(),
            bagimliliklar: vec![a, 42],
        };
        assert!(d.ekle(girdi, 1).is_err());
        assert_eq!(d.uzunluk(), 1, "hatali ekleme kismi olmamali");
    }

    #[test]
    fn siralama_oncelik_sure_kimlik() {
        let mut d = depo();
        let a = ekle(&mut d, "A", 60);
        let b = ekle(&mut d, "B", 10);
        let c = ekle(&mut d, "C", 30);
        d.getir_mut(c).unwrap().oncelik = Oncelik::Yuksek;
        d.getir_mut(b).unwrap().oncelik = Oncelik::Yuksek;
        let sira: Vec<u32> = d.sirali().iter().map(|g| g.id).collect();
        assert_eq!(sira, vec![b, c, a], "yuksek oncelik once, sonra kisa sure");
    }

    #[test]
    fn durum_ve_etiket_filtresi_calisir() {
        let mut d = depo();
        let a = d
            .ekle(
                GorevGirdi {
                    baslik: "A".to_string(),
                    tahmini_dakika: 10,
                    oncelik: Oncelik::Normal,
                    etiketler: vec!["Rapor".to_string()],
                    bagimliliklar: Vec::new(),
                },
                1,
            )
            .unwrap();
        d.durum_degistir(a, Durum::Ertelendi, 2).unwrap();
        assert_eq!(d.duruma_gore(Durum::Ertelendi).len(), 1);
        assert_eq!(d.etikete_gore("rapor").unwrap().len(), 1);
        assert_eq!(d.etiket_sayisi("#Rapor").unwrap(), 1);
        assert_eq!(d.etikete_gore("yok").unwrap().len(), 0);
    }
}
