//! Zaman ve takvim aritmetigi: UTC epoch saniyesi + kullanici saat dilimi ofseti.
//!
//! `chrono` ve `time` crate'leri bagimlilik politikasi nedeniyle yasaktir
//! (WORKER_CONTRACT.md 3.2-F). Bu yuzden sivil tarih donusumu elle yazildi:
//! Howard Hinnant'in kamu mali "chrono-compatible Low-Level Date Algorithms"
//! yontemi (1970-01-01 tabanli gun sayimi) saf tam sayi aritmetigi olarak
//! uygulandi. Yaz saati ve yaz gunesi kurali bilincli olarak modellenmedi:
//! kullanici sabit bir ofset (`--tz-offset`) verir, boylece oturum suresi
//! sistem saatindeki duzeltmelerden etkilenmez.

use crate::hata::{Hata, Sonuc};

/// Bir gundeki saniye sayisi.
pub const GUN_SANIYE: i64 = 86_400;

/// 1970-01-01 tarihinin gun numarasi (Hinnant algoritmasinin referans noktasi).
const REFERANS_GUN: i64 = 0;

/// Haftanin gun adlari, Pazartesi'den Pazar'a.
const GUN_ADLARI: [&str; 7] = [
    "Pazartesi",
    "Sali",
    "Carsamba",
    "Persembe",
    "Cuma",
    "Cumartesi",
    "Pazar",
];

/// Sabit ofsetli bir kullanici saat dilimi.
///
/// Ofset dakika cinsindendir: Turkiye `180`, `-05:00` `-300` demektir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaatDilimi {
    ofset_dakika: i32,
}

impl SaatDilimi {
    /// Verilen ofsetle (dakika) bir saat dilimi olusturur.
    ///
    /// Yalnizca `-840..=840` (-14:00 .. +14:00) araligi kabul edilir; gercek
    /// hayatta gunde 24 saatten fazla ofset olmadigi icin bu bir tutarlilik denetimidir.
    pub fn yeni(ofset_dakika: i32) -> Sonuc<Self> {
        if !(-14 * 60..=14 * 60).contains(&ofset_dakika) {
            return Err(Hata::GecersizSaatDilimi { ofset_dakika });
        }
        Ok(Self { ofset_dakika })
    }

    /// UTC'ye gore ofseti (dakika) dondurur.
    pub fn ofset_dakika(&self) -> i32 {
        self.ofset_dakika
    }

    /// Epoch saniyesini, kullanici yerelinde "1970-01-01'den kac gun gecti" sayisina cevirir.
    pub fn gun_no(&self, epoch: i64) -> i64 {
        (epoch + i64::from(self.ofset_dakika) * 60).div_euclid(GUN_SANIYE)
    }

    /// Epoch'un ait oldugu yerel gunun UTC epoch baslangicini dondurur.
    ///
    /// Gece yarisi siniri: tam `00:00:00` yerel zaman yeni gun sayilir,
    /// `23:59:59` ise bir onceki gunde kalir.
    pub fn gun_baslangic(&self, epoch: i64) -> i64 {
        self.gun_no(epoch) * GUN_SANIYE - i64::from(self.ofset_dakika) * 60
    }

    /// Verilen yerel gunun bitisini (bir sonraki gunun baslangicini) dondurur.
    pub fn gun_sonu(&self, epoch: i64) -> i64 {
        self.gun_baslangic(epoch) + GUN_SANIYE
    }

    /// Yerel gunun `haftan_gunu` degerini dondurur (0 = Pazartesi).
    pub fn haftan_gunu(&self, epoch: i64) -> usize {
        // 1970-01-1 Persembediydi; (gun_no + 4) % 7 ile 0 = Pazartesi elde edilir.
        (self.gun_no(epoch) + 4).rem_euclid(7) as usize
    }
}

/// Bu gunun UTC epoch saniyesini dondurur.
///
/// Sistem saati 1970 oncesine ayarlanmis bir makinede `0` doner; arac
/// kilitlenmez, yalnizca tarih damgasi 1970'e duser.
pub fn simdi_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|s| s.as_secs() as i64)
        .unwrap_or(0)
}

/// Sivil tarihi (yil, ay, gun) 1970-01-01 tabanli gun sayisina cevirir.
fn gun_sayisi(yil: i32, ay: u32, gun: u32) -> i64 {
    let ay = i64::from(ay);
    let gun = i64::from(gun);
    let mut y = i64::from(yil);
    if ay <= 2 {
        y -= 1;
    }
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = (ay + 9) % 12; // Mart = 0
    let doy = (153 * mp + 2) / 5 + gun - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + doe - REFERANS_GUN - 719_468
}

/// 1970-01-01 tabanli gun sayisini sivil tarihe (yil, ay, gun) cevirir.
fn sivil_tarih(gun_no: i64) -> (i32, u32, u32) {
    let z = gun_no + REFERANS_GUN + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11], Mart = 0
    let gun = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let ay = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let yil = if ay <= 2 { y + 1 } else { y } as i32;
    (yil, ay, gun)
}

/// Uc bileenden (yil, ay, gun) gecerli bir gun sayisi uretir.
///
/// Gecersiz tarihler (30 Subat, 13. ay, 0. gun) gidis-donus denetimiyle elenir.
pub fn gun_no_uret(yil: i32, ay: u32, gun: u32) -> Sonuc<i64> {
    if !(1..=12).contains(&ay) || gun == 0 || gun > 31 {
        return Err(Hata::GecersizTarih {
            girdi: format!("{yil}-{ay:02}-{gun:02}"),
        });
    }
    let sayi = gun_sayisi(yil, ay, gun);
    let (e_yil, e_ay, e_gun) = sivil_tarih(sayi);
    if e_yil != yil || e_ay != ay || e_gun != gun {
        return Err(Hata::GecersizTarih {
            girdi: format!("{yil}-{ay:02}-{gun:02}"),
        });
    }
    Ok(sayi)
}

/// `YYYY-MM-DD` bicimli metni sivil tarihe cevirir.
pub fn tarih_ayir(girdi: &str) -> Sonuc<(i32, u32, u32)> {
    let parcalar: Vec<&str> = girdi.split('-').collect();
    if parcalar.len() != 3 {
        return Err(Hata::GecersizTarih {
            girdi: girdi.to_string(),
        });
    }
    let yil = parcalar[0]
        .parse::<i32>()
        .map_err(|_| Hata::GecersizTarih {
            girdi: girdi.to_string(),
        })?;
    let ay = parcalar[1]
        .parse::<u32>()
        .map_err(|_| Hata::GecersizTarih {
            girdi: girdi.to_string(),
        })?;
    let gun = parcalar[2]
        .parse::<u32>()
        .map_err(|_| Hata::GecersizTarih {
            girdi: girdi.to_string(),
        })?;
    if parcalar[0].len() != 4 || parcalar[1].len() != 2 || parcalar[2].len() != 2 {
        return Err(Hata::GecersizTarih {
            girdi: girdi.to_string(),
        });
    }
    // Ayrik dogrulama: gun_no_uret gecersiz tarihleri zaten eliyor.
    gun_no_uret(yil, ay, gun)?;
    Ok((yil, ay, gun))
}

/// Gun sayisini `YYYY-MM-DD` metnine cevirir.
pub fn tarih_yaz(gun_no: i64) -> String {
    let (yil, ay, gun) = sivil_tarih(gun_no);
    format!("{yil:04}-{ay:02}-{gun:02}")
}

/// Yerel tarih ve saati epoch'a cevirir; saat dilimi ofseti geri uygulanir.
pub fn yerel_epoch_uret(
    yil: i32,
    ay: u32,
    gun: u32,
    saat: i32,
    dakika: i32,
    saat_dilimi: &SaatDilimi,
) -> Sonuc<i64> {
    if !(0..24).contains(&saat) || !(0..60).contains(&dakika) {
        return Err(Hata::GecersizTarih {
            girdi: format!("{yil}-{ay:02}-{gun:02} {saat:02}:{dakika:02}"),
        });
    }
    let gun_no = gun_no_uret(yil, ay, gun)?;
    let yerel = gun_no * GUN_SANIYE + i64::from(saat) * 3600 + i64::from(dakika) * 60;
    Ok(yerel - i64::from(saat_dilimi.ofset_dakika) * 60)
}

/// `YYYY-MM-DD HH:MM` metnini epoch'a cevirir (kullanici yerel saati).
pub fn yerel_metin_ayir(girdi: &str, saat_dilimi: &SaatDilimi) -> Sonuc<i64> {
    let parcalar: Vec<&str> = girdi.split(' ').collect();
    if parcalar.len() != 2 || parcalar[1].len() != 5 {
        return Err(Hata::GecersizTarih {
            girdi: girdi.to_string(),
        });
    }
    let (yil, ay, gun) = tarih_ayir(parcalar[0])?;
    let saat_dakika: Vec<&str> = parcalar[1].split(':').collect();
    if saat_dakika.len() != 2 {
        return Err(Hata::GecersizTarih {
            girdi: girdi.to_string(),
        });
    }
    let saat = saat_dakika[0]
        .parse::<i32>()
        .map_err(|_| Hata::GecersizTarih {
            girdi: girdi.to_string(),
        })?;
    let dakika = saat_dakika[1]
        .parse::<i32>()
        .map_err(|_| Hata::GecersizTarih {
            girdi: girdi.to_string(),
        })?;
    yerel_epoch_uret(yil, ay, gun, saat, dakika, saat_dilimi)
}

/// Epoch saniyesini RFC 3339 / ISO 8601 UTC metnine cevirir.
///
/// Bicim: `YYYY-MM-DDTHH:MM:SSZ`. Kaynak: RFC 3339 bolum 5.6 (date-time bicimi).
pub fn iso8601_utc(epoch: i64) -> String {
    let gun_no = epoch.div_euclid(GUN_SANIYE);
    let kalan = epoch.rem_euclid(GUN_SANIYE);
    let (yil, ay, gun) = sivil_tarih(gun_no);
    format!(
        "{yil:04}-{ay:02}-{gun:02}T{:02}:{:02}:{:02}Z",
        kalan / 3600,
        (kalan % 3600) / 60,
        kalan % 60
    )
}

/// Epoch saniyesini kullanici yerelinde `YYYY-MM-DD HH:MM` bicimine cevirir.
pub fn yerel_zaman_yaz(epoch: i64, saat_dilimi: &SaatDilimi) -> String {
    let yerel = epoch + i64::from(saat_dilimi.ofset_dakika) * 60;
    let gun_no = yerel.div_euclid(GUN_SANIYE);
    let kalan = yerel.rem_euclid(GUN_SANIYE);
    format!(
        "{} {:02}:{:02}",
        tarih_yaz(gun_no),
        kalan / 3600,
        (kalan % 3600) / 60
    )
}

/// Gun sayisinin ISO-8601 yil ve hafta numarasini dondurur.
///
/// ISO-8601 kurali: hafta, Perşembeye duygun yone sahiptir; yil, 4 Ocak ve
/// onceki gunun Perşembesine duygun yilde degilse hafta, bir onceki yila aittir.
pub fn iso_yil_ve_hafta(gun_no: i64) -> (i32, u32) {
    // ISO gun indeksi: 0 = Pazartesi ... 6 = Pazar (1970-01-1 = Persembe = 3)
    let iso_indeks = (gun_no + 3).rem_euclid(7);
    let pazartesi = gun_no - iso_indeks;
    let perembe = pazartesi + 3;
    let (yil, _, _) = sivil_tarih(perembe);
    let hafta = (perembe - yilin_ilk_perembesi(yil)) / 7 + 1;
    (yil, hafta as u32)
}

/// Verilen yilin, ISO-8601'ye gore 1. haftasinin Perşembe gununun gun sayisi.
fn yilin_ilk_perembesi(yil: i32) -> i64 {
    let ocak_bir = gun_sayisi(yil, 1, 1);
    let ocak_bir_haftan_gunu = (ocak_bir + 3).rem_euclid(7);
    ocak_bir + (3 - ocak_bir_haftan_gunu).rem_euclid(7)
}

/// Verilen yilin 1. ISO haftasinin Pazartesi gun sayisi.
fn yilin_ilk_pazartesisi(yil: i32) -> i64 {
    yilin_ilk_perembesi(yil) - 3
}

/// Gun sayisini `YYYY-Www` metnine cevirir.
pub fn hafta_yaz(gun_no: i64) -> String {
    let (yil, hafta) = iso_yil_ve_hafta(gun_no);
    format!("{yil}-W{hafta:02}")
}

/// `YYYY-Www` metnini, o haftanin Pazartesi gun sayisina cevirir.
pub fn hafta_ayir(girdi: &str) -> Sonuc<i64> {
    let (yil, hafta_no) = hafta_bilesenleri_ayir(girdi)?;
    Ok(pazartesi_bul(yil, hafta_no))
}

/// `YYYY-Www` metnini (yil, hafta_no) ciftine cevirir; hafta numarasi 1..=53 olmalidir.
pub fn hafta_ayir_yil_hafta(girdi: &str) -> Sonuc<(i32, u32)> {
    hafta_bilesenleri_ayir(girdi)
}

/// `YYYY-Www` metnini yil ve hafta numarasi olarak ayristirir.
///
/// `W` oneki zorunludur; `2026-40` bir hafta degil, gecersiz bir girdidir.
fn hafta_bilesenleri_ayir(girdi: &str) -> Sonuc<(i32, u32)> {
    let buyuk_kisim: Vec<&str> = girdi.split('-').collect();
    if buyuk_kisim.len() != 2 {
        return Err(Hata::GecersizHafta {
            girdi: girdi.to_string(),
        });
    }
    let (yil_metin, hafta_metin) = (buyuk_kisim[0], buyuk_kisim[1]);
    if yil_metin.len() != 4 {
        return Err(Hata::GecersizHafta {
            girdi: girdi.to_string(),
        });
    }
    let onek_kaldirilmis = hafta_metin.strip_prefix(['W', 'w']);
    let takvim = match onek_kaldirilmis {
        Some(kalan) if kalan.len() == 2 => kalan,
        _ => {
            return Err(Hata::GecersizHafta {
                girdi: girdi.to_string(),
            })
        }
    };
    let yil = yil_metin.parse::<i32>().map_err(|_| Hata::GecersizHafta {
        girdi: girdi.to_string(),
    })?;
    let hafta_no = takvim.parse::<u32>().map_err(|_| Hata::GecersizHafta {
        girdi: girdi.to_string(),
    })?;
    if !(1..=53).contains(&hafta_no) {
        return Err(Hata::GecersizHafta {
            girdi: girdi.to_string(),
        });
    }
    Ok((yil, hafta_no))
}

/// ISO yil/hafta ciftinden o haftanin Pazartesi gun sayisini hesaplar.
fn pazartesi_bul(yil: i32, hafta_no: u32) -> i64 {
    yilin_ilk_pazartesisi(yil) + i64::from(hafta_no - 1) * 7
}

/// Bir ISO haftasindaki yedi gunun gun numaralarini Pazartesi'den Pazar'a dondurur.
pub fn hafta_gunleri(hafta_baslangic_gun_no: i64) -> Vec<i64> {
    (0..7).map(|g| hafta_baslangic_gun_no + g).collect()
}

/// Gun sayisinin Turkce gun adini dondurur.
pub fn gun_adi(gun_no: i64) -> &'static str {
    let indeks = (gun_no + 3).rem_euclid(7) as usize;
    GUN_ADLARI[indeks]
}

/// Dakikayi `2 sa 15 dk` gibi okunur bir metne cevirir.
pub fn sure_yaz(dakika: i32) -> String {
    let isaretli = dakika < 0;
    let mut kalan = i64::from(dakika).abs();
    let saat = kalan / 60;
    kalan %= 60;
    let govde = if saat > 0 && kalan > 0 {
        format!("{saat} sa {kalan} dk")
    } else if saat > 0 {
        format!("{saat} sa")
    } else {
        format!("{kalan} dk")
    };
    if isaretli {
        format!("-{govde}")
    } else {
        govde
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;

    fn tr() -> SaatDilimi {
        SaatDilimi::yeni(180).unwrap()
    }

    #[test]
    fn referans_gun_sifirdir() {
        assert_eq!(tarih_yaz(0), "1970-01-01");
        assert_eq!(iso8601_utc(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn bilinen_tarihler_gidis_donus_yapar() {
        for metin in [
            "1970-01-01",
            "2000-02-29",
            "2026-01-01",
            "2026-09-29",
            "2100-12-31",
            "1999-12-31",
        ] {
            let (y, a, g) = tarih_ayir(metin).unwrap();
            let sayi = gun_no_uret(y, a, g).unwrap();
            assert_eq!(tarih_yaz(sayi), metin, "gidis-donus: {metin}");
        }
    }

    #[test]
    fn gecersiz_tarihler_elenir() {
        for bozuk in [
            "2026-02-30",
            "2026-13-01",
            "2026-00-10",
            "2026-01-00",
            "2025-02-29",
        ] {
            assert!(
                tarih_ayir(bozuk).is_err(),
                "reddedilmesi bekleniyordu: {bozuk}"
            );
        }
    }

    #[test]
    fn tarih_bicim_kurallari_uygulanir() {
        assert!(tarih_ayir("2026-9-29").is_err());
        assert!(tarih_ayir("26-09-29").is_err());
        assert!(tarih_ayir("2026/09/29").is_err());
    }

    #[test]
    fn gun_siniri_gece_yari_yeni_gundur() {
        let tz = tr();
        // 2026-09-29 00:00:00 yerel
        let gece_yari = yerel_epoch_uret(2026, 9, 29, 0, 0, &tz).unwrap();
        assert_eq!(tarih_yaz(tz.gun_no(gece_yari)), "2026-09-29");
        // bir saniye once hala onceki gun
        assert_eq!(
            tarih_yaz(tz.gun_no(gece_yari - 1)),
            "2026-09-28",
            "gece yarisi siniri"
        );
        // bir saniye sonra ayni gun
        assert_eq!(tarih_yaz(tz.gun_no(gece_yari + 1)), "2026-09-29");
    }

    #[test]
    fn gun_siniri_gun_baslangic_ve_sonu_tutarli() {
        let tz = tr();
        let oz = yerel_epoch_uret(2026, 9, 29, 14, 37, &tz).unwrap();
        assert_eq!(
            yerel_zaman_yaz(tz.gun_baslangic(oz), &tz),
            "2026-09-29 00:00"
        );
        assert_eq!(yerel_zaman_yaz(tz.gun_sonu(oz), &tz), "2026-09-30 00:00");
        assert_eq!(tz.gun_sonu(oz) - tz.gun_baslangic(oz), GUN_SANIYE);
    }

    #[test]
    fn saat_dilimi_ofseti_gun_degistirir() {
        let utc = SaatDilimi::yeni(0).unwrap();
        let tr = tr();
        // 2026-09-29 22:00 UTC = 2026-09-30 01:00 TR
        let epoch = gun_no_uret(2026, 9, 29).unwrap() * GUN_SANIYE + 22 * 3600;
        assert_eq!(tarih_yaz(utc.gun_no(epoch)), "2026-09-29");
        assert_eq!(tarih_yaz(tr.gun_no(epoch)), "2026-09-30");
        assert_eq!(yerel_zaman_yaz(epoch, &tr), "2026-09-30 01:00");
    }

    #[test]
    fn gecersiz_saat_dilimi_reddedilir() {
        assert!(SaatDilimi::yeni(841).is_err());
        assert!(SaatDilimi::yeni(-841).is_err());
        assert!(SaatDilimi::yeni(840).is_ok());
        assert!(SaatDilimi::yeni(-840).is_ok());
    }

    #[test]
    fn iso_hafta_yil_dongusu_dogru() {
        // 2026-01-1 Perşembe: 2026'nin 1. haftası
        let gun = gun_no_uret(2026, 1, 1).unwrap();
        assert_eq!(iso_yil_ve_hafta(gun), (2026, 1));
        assert_eq!(hafta_yaz(gun), "2026-W01");
        // 2027-01-01 Cuma: 2026'nin 53. haftasına ait
        let gun = gun_no_uret(2027, 1, 1).unwrap();
        assert_eq!(hafta_yaz(gun), "2026-W53");
        // 2026-09-29 Pazartesi
        let gun = gun_no_uret(2026, 9, 29).unwrap();
        assert_eq!(hafta_yaz(gun), "2026-W40");
    }

    #[test]
    fn hafta_ayir_gidis_donus_yapar() {
        let (yil, hafta) = hafta_ayir_yil_hafta("2026-W40").unwrap();
        assert_eq!((yil, hafta), (2026, 40));
        let pazartesi = hafta_ayir("2026-W40").unwrap();
        assert_eq!(tarih_yaz(pazartesi), "2026-09-28");
        assert_eq!(hafta_yaz(pazartesi), "2026-W40");
    }

    #[test]
    fn gecersiz_hafta_reddedilir() {
        for bozuk in [
            "2026-40", "2026-W00", "2026-W54", "26-W10", "2026-W1", "2026-W1a",
        ] {
            assert!(hafta_ayir(bozuk).is_err(), "reddedilmeli: {bozuk}");
        }
    }

    #[test]
    fn hafta_yedi_gun_doner() {
        let gunler = hafta_gunleri(hafta_ayir("2026-W40").unwrap());
        assert_eq!(gunler.len(), 7);
        assert_eq!(tarih_yaz(gunler[0]), "2026-09-28");
        assert_eq!(tarih_yaz(gunler[6]), "2026-10-04");
    }

    #[test]
    fn gun_adi_pazartesi_baslar() {
        let pazartesi = gun_no_uret(2026, 9, 28).unwrap();
        assert_eq!(gun_adi(pazartesi), "Pazartesi");
        assert_eq!(gun_adi(pazartesi + 6), "Pazar");
        assert_eq!(gun_adi(pazartesi + 1), "Sali");
    }

    #[test]
    fn sure_yaz_kisa_ve_uzun_bicim() {
        assert_eq!(sure_yaz(0), "0 dk");
        assert_eq!(sure_yaz(45), "45 dk");
        assert_eq!(sure_yaz(60), "1 sa");
        assert_eq!(sure_yaz(135), "2 sa 15 dk");
        assert_eq!(sure_yaz(-30), "-30 dk");
    }

    #[test]
    fn negatif_gun_no_cekirilir() {
        // 1969-12-31, 1970-01-01'den bir gun once
        assert_eq!(tarih_yaz(-1), "1969-12-31");
        assert_eq!(iso8601_utc(-1), "1969-12-31T23:59:59Z");
    }

    #[test]
    fn iso8601_utc_zaman_damgasini_yazar() {
        let tz = tr();
        let epoch = yerel_epoch_uret(2026, 9, 29, 15, 4, &tz).unwrap();
        assert_eq!(iso8601_utc(epoch), "2026-09-29T12:04:00Z");
    }
}
