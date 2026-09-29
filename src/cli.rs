//! Terminal arayuzu: alt komut tanimlari ve komut yurutucusu.
//!
//! Grafik arayuz (WebView / pencere katmani) bagimlilik politikasiyla kalici
//! olarak yasaktir (KARAR D-010). Bu yuzden tum cikti duz metindir ve stdout'a
//! yazilir; mantik `gorevleri_oku` / `gorevleri_yaz` gibi cekirdek API'ler
//! uzerinden gecer, boylece testlerde dogrudan dogrulanabilir.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::depo::Depo;
use crate::gorev::{Durum, Gorev, GorevGirdi, Oncelik};
use crate::hata::{Hata, Sonuc};
use crate::kapasite::{planla, GunlukKapasite};
use crate::oturum::{Oturum, OturumGunlugu, OturumTuru};
use crate::rapor::rapor_olustur;
use crate::zaman::{
    hafta_ayir, iso8601_utc, simdi_epoch, sure_yaz, tarih_ayir, tarih_yaz, SaatDilimi, GUN_SANIYE,
};

/// Pomodoro, gorev bagimliligi ve gunluk kapasite planlayan cevrimdisi arac.
#[derive(Debug, Parser)]
#[command(
    name = "focuscompass",
    version,
    about = "Pomodoro oturumu, gorev bagimliligi ve gunluk kapasite planlayan cevrimdisi arac",
    long_about = "FocusCompass (PusulaVakti): pomodoro oturumu, gorev bagimliligi cozumu ve \
                  gunluk kapasite planlamasini birlikte sunan terminal araci. Tum veri yerel \
                  dosyalarda tutulur; hicbir ag baglantisi acilmaz."
)]
pub struct Cli {
    /// Veri klasoru. Varsayilan: calisma dizinindeki `veri/`.
    #[arg(long, global = true, value_name = "YOL")]
    pub veri: Option<PathBuf>,

    /// Kullanici saat dilimi ofseti (dakika). Turkiye icin 180.
    #[arg(long, global = true, value_name = "DAKIKA", default_value_t = 180)]
    pub tz_offset: i32,

    /// Calistirilacak alt komut.
    #[command(subcommand)]
    pub komut: Komut,
}

/// Oturum turu secenekleri (`clap` deger denetimi burada yapilir).
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TurSecimi {
    /// Derin calisma oturumu.
    Odak,
    /// Kisa mola.
    Kisa,
    /// Uzun mola.
    Uzun,
}

impl From<TurSecimi> for OturumTuru {
    fn from(deger: TurSecimi) -> Self {
        match deger {
            TurSecimi::Odak => Self::Odak,
            TurSecimi::Kisa => Self::KisaMola,
            TurSecimi::Uzun => Self::UzunMola,
        }
    }
}

/// Alt komutlar.
#[derive(Debug, Subcommand)]
pub enum Komut {
    /// Yeni gorev ekler.
    Add(EkleArg),
    /// Gorevleri listeler.
    List(ListeArg),
    /// Bir gorevde pomodoro oturumu baslatir.
    Start(BaslatArg),
    /// Acik oturumu kapatir ve gorevi tamamlar.
    Stop(DurdurArg),
    /// Acik oturuma kesinti kaydeder.
    Interrupt(KesintiArg),
    /// Bagimlilik yonetimi.
    #[command(subcommand)]
    Dep(BagimlilikKomut),
    /// Bir veya birden fazla pomodoro dongusu calistirir.
    Cycle(DonguArg),
    /// Gunluk kapasite planini uretir.
    Plan(PlanArg),
    /// Haftalik raporu JSON ve Markdown olarak yazar.
    Report(RaporArg),
}

/// `add` alt komutunun argumanlari.
#[derive(Debug, Args)]
pub struct EkleArg {
    /// Gorev basliklari; birden fazla verilirse her biri ayri gorev olur.
    #[arg(required = true, value_name = "BASLIK")]
    pub basliklar: Vec<String>,
    /// Tahmini sure (dakika).
    #[arg(long, short = 'm', default_value_t = 25, value_name = "DAKIKA")]
    pub dakika: i32,
    /// Oncelik: dusuk | normal | yuksek.
    #[arg(long, value_name = "SEVIYE", default_value = "normal")]
    pub oncelik: String,
    /// Tekrarlanabilir etiket.
    #[arg(long = "tag", short = 't', value_name = "ETIKET")]
    pub etiket: Vec<String>,
    /// Bu gorevin bitmeden once bitmesi gereken gorev kimligi.
    #[arg(long = "depends", short = 'd', value_name = "ID")]
    pub bagimlilik: Vec<u32>,
}

/// `list` alt komutunun argumanlari.
#[derive(Debug, Args)]
pub struct ListeArg {
    /// Durum filtresi: bekliyor | calisiyor | tamamlandi | ertelendi.
    #[arg(long, short = 's', value_name = "DURUM")]
    pub durum: Option<String>,
    /// Etiket filtresi.
    #[arg(long = "tag", short = 't', value_name = "ETIKET")]
    pub etiket: Option<String>,
    /// Yalnizca kimlik ve baslik goster.
    #[arg(long)]
    pub ozet: bool,
}

/// `start` alt komutunun argumanlari.
#[derive(Debug, Args)]
pub struct BaslatArg {
    /// Baslatilacak gorevin kimligi.
    pub id: u32,
    /// Oturumun planlanan suresi (dakika).
    #[arg(long, short = 'm', default_value_t = 25, value_name = "DAKIKA",
          value_parser = clap::value_parser!(i32).range(1..=240))]
    pub dakika: i32,
    /// Oturum turu.
    #[arg(long, value_enum, default_value = "odak", value_name = "TUR")]
    pub tur: TurSecimi,
}

/// `stop` alt komutunun argumanlari.
#[derive(Debug, Args)]
pub struct DurdurArg {
    /// Gorevi tamamlamak yerine ertele.
    #[arg(long)]
    pub ertele: bool,
    /// Oturumun bitecegi zamani UTC epoch saniyesi olarak ver.
    #[arg(long, value_name = "EPOCH")]
    pub bitis: Option<i64>,
}

/// `interrupt` alt komutunun argumanlari.
#[derive(Debug, Args)]
pub struct KesintiArg {
    /// Kesintinin kaynagi (kisa etiket, ornegin `toplanti`).
    pub kaynak: String,
    /// Kesintinin suresi (dakika).
    #[arg(long, short = 'm', value_name = "DAKIKA",
          value_parser = clap::value_parser!(i32).range(1..=240))]
    pub dakika: i32,
    /// Kesintinin olustugu zamani UTC epoch saniyesi olarak ver.
    #[arg(long, value_name = "EPOCH")]
    pub simdi: Option<i64>,
}

/// `dep` alt komutlari.
#[derive(Debug, Subcommand)]
pub enum BagimlilikKomut {
    /// Bagimlilik ekler; dongu olusturursa reddedilir.
    Ekle(BagimlilikCiftArg),
    /// Bagimliligi kaldirir.
    Kaldir(BagimlilikCiftArg),
    /// Gecisli bagimlilik agacini gosterir.
    Agac(AgacArg),
}

/// Iki gorev kimliginden olusan arguman.
#[derive(Debug, Args)]
pub struct BagimlilikCiftArg {
    /// Bagimlilik eklenen (veya kaldirilan) gorev.
    pub gorev: u32,
    /// Bagimlilik olan gorev.
    pub bagimlilik: u32,
}

/// `dep agac` argumanlari.
#[derive(Debug, Args)]
pub struct AgacArg {
    /// Koku gorevin kimligi.
    pub gorev: u32,
}

/// `cycle` alt komutunun argumanlari.
#[derive(Debug, Args)]
pub struct DonguArg {
    /// Calisilacak gorevin kimligi.
    pub id: u32,
    /// Odak suresi (dakika).
    #[arg(long = "odak-dk", default_value_t = 25, value_name = "DAKIKA",
          value_parser = clap::value_parser!(i32).range(1..=180))]
    pub odak_dakika: i32,
    /// Mola suresi (dakika).
    #[arg(long = "mola-dk", default_value_t = 5, value_name = "DAKIKA",
          value_parser = clap::value_parser!(i32).range(0..=120))]
    pub mola_dakika: i32,
    /// Tekrar sayisi.
    #[arg(long, short = 'n', default_value_t = 1, value_name = "ADET",
          value_parser = clap::value_parser!(i32).range(1..=12))]
    pub dongu: i32,
    /// Mola turu.
    #[arg(
        long = "mola-tipi",
        value_enum,
        default_value = "kisa",
        value_name = "TUR"
    )]
    pub mola_tipi: TurSecimi,
    /// Her odak oturumundan sonra kaydedilecek kesinti (dakika).
    #[arg(long = "kesinti-dk", value_name = "DAKIKA",
          value_parser = clap::value_parser!(i32).range(0..=240))]
    pub kesinti_dakika: Option<i32>,
    /// Kesinti kaynagi.
    #[arg(
        long = "kesinti-kaynak",
        default_value = "toplanti",
        value_name = "KAYNAK"
    )]
    pub kesinti_kaynak: String,
    /// Beklemeden kayit tut: oturumlar planlanan sureyle yazilir.
    #[arg(long)]
    pub kuru: bool,
}

/// `plan` alt komutunun argumanlari.
#[derive(Debug, Args)]
pub struct PlanArg {
    /// Gunluk derin calisma kapasitesi (dakika).
    #[arg(long, short = 'c', default_value_t = 180, value_name = "DAKIKA")]
    pub kapasite: i32,
    /// Toplanti/ulasim gibi sabit sureler (dakika).
    #[arg(long, short = 'b', default_value_t = 0, value_name = "DAKIKA")]
    pub sabit: i32,
    /// Planlanacak tarih (YYYY-MM-DD). Varsayilan: bugun.
    #[arg(long, value_name = "TARIH")]
    pub tarih: Option<String>,
}

/// `report` alt komutunun argumanlari.
#[derive(Debug, Args)]
pub struct RaporArg {
    /// ISO haftasi (YYYY-Www). Varsayilan: bulunulan hafta.
    #[arg(long, value_name = "HAFTA")]
    pub hafta: Option<String>,
    /// Cikti bicimi: json | md | ikisi.
    #[arg(long, default_value = "ikisi", value_name = "BI CIM")]
    pub format: String,
    /// Raporun yazilacagi veri klasoru (global `--veri` yerine gecer).
    #[arg(long, value_name = "YOL")]
    pub cikti: Option<PathBuf>,
}

/// Ayristirilmis komut baglami: veri klasoru, saat dilimi ve simdiki an.
struct Baglam {
    depo: Depo,
    saat_dilimi: SaatDilimi,
    simdi: i64,
}

impl Baglam {
    fn olustur(cli: &Cli) -> Sonuc<Self> {
        let saat_dilimi = SaatDilimi::yeni(cli.tz_offset)?;
        let kok = cli.veri.clone().unwrap_or_else(|| PathBuf::from("veri"));
        let depo = Depo::yeni(kok);
        depo.hazirla()?;
        Ok(Self {
            depo,
            saat_dilimi,
            simdi: simdi_epoch(),
        })
    }

    /// Bugunun tarihi (`YYYY-MM-DD`), kullanici saat diliminde.
    fn bugun(&self) -> String {
        tarih_yaz(self.saat_dilimi.gun_no(self.simdi))
    }

    /// Oturum gunlugunu dosyadan yukler (ayni sira icin son kayit gecerlidir).
    fn gunluk(&self) -> Sonuc<OturumGunlugu> {
        Ok(OturumGunlugu::yukle(self.depo.oturumlari_oku()?))
    }
}

/// Komutu yurutur ve yazdirilacak metni dondurur.
///
/// Hata `Sonuc` ile yukari cikar; `main` hatayi `stderr`'e yazar ve `1` ile cikar.
pub fn calistir(cli: &Cli) -> Sonuc<String> {
    match &cli.komut {
        Komut::Add(arg) => gorev_ekle(cli, arg),
        Komut::List(arg) => gorev_listele(cli, arg),
        Komut::Start(arg) => oturum_baslat(cli, arg),
        Komut::Stop(arg) => oturum_durdur(cli, arg),
        Komut::Interrupt(arg) => kesinti_ekle(cli, arg),
        Komut::Dep(komut) => bagimlilik_calistir(cli, komut),
        Komut::Cycle(arg) => dongu_calistir(cli, arg),
        Komut::Plan(arg) => plan_uret(cli, arg),
        Komut::Report(arg) => rapor_uret(cli, arg),
    }
}

fn gorev_ekle(cli: &Cli, arg: &EkleArg) -> Sonuc<String> {
    let baglam = Baglam::olustur(cli)?;
    let oncelik = Oncelik::ayir(&arg.oncelik)?;
    let mut depo = baglam.depo.gorevleri_oku()?;
    let mut satirlar: Vec<String> = Vec::new();
    for baslik in &arg.basliklar {
        let id = depo.ekle(
            GorevGirdi {
                baslik: baslik.clone(),
                tahmini_dakika: arg.dakika,
                oncelik,
                etiketler: arg.etiket.clone(),
                bagimliliklar: arg.bagimlilik.clone(),
            },
            baglam.simdi,
        )?;
        let gorev = depo.getir(id)?;
        satirlar.push(format!(
            "eklendi #{id}  {}  {}  oncelik={}  durum={}{}{}",
            gorev.baslik,
            sure_yaz(gorev.tahmini_dakika),
            gorev.oncelik.ad(),
            gorev.durum.ad(),
            etiket_ek(gorev),
            bagimlilik_ek(gorev)
        ));
    }
    baglam.depo.gorevleri_yaz(&depo, baglam.simdi)?;
    Ok(satirlar.join("\n"))
}

fn etiket_ek(gorev: &Gorev) -> String {
    if gorev.etiketler.is_empty() {
        String::new()
    } else {
        format!("  etiket={}", gorev.etiketler.join(","))
    }
}

fn bagimlilik_ek(gorev: &Gorev) -> String {
    if gorev.bagimliliklar.is_empty() {
        String::new()
    } else {
        let liste: Vec<String> = gorev
            .bagimliliklar
            .iter()
            .map(|k| format!("#{k}"))
            .collect();
        format!("  bagimlilik={}", liste.join("+"))
    }
}

fn gorev_listele(cli: &Cli, arg: &ListeArg) -> Sonuc<String> {
    let baglam = Baglam::olustur(cli)?;
    let depo = baglam.depo.gorevleri_oku()?;
    let durum = match &arg.durum {
        Some(metin) => Some(Durum::ayir(metin)?),
        None => None,
    };
    let etiketli: Option<Vec<u32>> = match &arg.etiket {
        Some(etiket) => Some(depo.etikete_gore(etiket)?.iter().map(|g| g.id).collect()),
        None => None,
    };
    let filtre: Vec<&Gorev> = depo
        .sirali()
        .into_iter()
        .filter(|g| match durum {
            Some(d) => g.durum == d,
            None => true,
        })
        .filter(|g| match &etiketli {
            Some(liste) => liste.contains(&g.id),
            None => true,
        })
        .collect();
    if filtre.is_empty() {
        return Ok("kayitli gorev yok".to_string());
    }
    let toplam = filtre.len();
    let mut satirlar: Vec<String> = Vec::new();
    for gorev in filtre {
        if arg.ozet {
            satirlar.push(format!("#{:<4} {}", gorev.id, gorev.baslik));
            continue;
        }
        satirlar.push(format!(
            "#{:<4} {:<11} {:<7} {:>4} dk  {}{}{}",
            gorev.id,
            gorev.durum.ad(),
            gorev.oncelik.ad(),
            gorev.tahmini_dakika,
            gorev.baslik,
            etiket_ek(gorev),
            bagimlilik_ek(gorev)
        ));
    }
    satirlar.push(format!("{toplam} gorev"));
    Ok(satirlar.join("\n"))
}

fn oturum_baslat(cli: &Cli, arg: &BaslatArg) -> Sonuc<String> {
    let baglam = Baglam::olustur(cli)?;
    let mut depo = baglam.depo.gorevleri_oku()?;
    let mut gunluk = baglam.gunluk()?;
    let tur: OturumTuru = arg.tur.into();
    if tur.odak_mi() {
        depo.getir(arg.id)?;
        depo.baslatilabilir(arg.id)?;
        depo.durum_degistir(arg.id, Durum::Calisiyor, baglam.simdi)?;
    }
    let sira = gunluk.baslat(Some(arg.id), tur, arg.dakika, baglam.simdi)?;
    baglam.depo.oturum_ekle(gunluk.getir(sira)?)?;
    baglam.depo.gorevleri_yaz(&depo, baglam.simdi)?;
    let gorev = depo.getir(arg.id)?;
    Ok(format!(
        "oturum #{sira} basladi: {}  planlanan {}  gorev=#{} {}  gunluk={}",
        tur.ad(),
        sure_yaz(arg.dakika),
        arg.id,
        gorev.baslik,
        baglam.depo.oturum_yolu().display()
    ))
}

fn oturum_durdur(cli: &Cli, arg: &DurdurArg) -> Sonuc<String> {
    let baglam = Baglam::olustur(cli)?;
    let mut depo = baglam.depo.gorevleri_oku()?;
    let mut gunluk = baglam.gunluk()?;
    let sira = gunluk
        .acik_oturum()
        .map(|o| o.sira)
        .ok_or(Hata::AcikOturumYok)?;
    let gorev_id = gunluk.getir(sira)?.gorev_id;
    let bitis = arg.bitis.unwrap_or(baglam.simdi);
    gunluk.getir_mut(sira)?.bitir(bitis)?;
    let kapali = gunluk.getir(sira)?.clone();

    let mut durum_mesaji = String::new();
    if let Some(id) = gorev_id {
        let hedef = if arg.ertele {
            Durum::Ertelendi
        } else {
            Durum::Tamamlandi
        };
        depo.durum_degistir(id, hedef, baglam.simdi)?;
        durum_mesaji = format!("  gorev=#{id} durum={}", hedef.ad());
    }

    // Gunluk eklenebilir dosyadir: kapanan oturumun yeni hali son satir olarak
    // yazilir, `OturumGunlugu::yukle` ayni sira icin son kaydi gecerli sayar.
    baglam.depo.oturum_ekle(&kapali)?;
    baglam.depo.gorevleri_yaz(&depo, baglam.simdi)?;
    gunluk_satir_ekle(&baglam, &kapali)?;
    Ok(format!(
        "oturum #{sira} kapandi  {}  kesinti={}{}",
        kapali.ozet()?,
        sure_yaz(kapali.toplam_kesinti()),
        durum_mesaji
    ))
}

fn kesinti_ekle(cli: &Cli, arg: &KesintiArg) -> Sonuc<String> {
    let baglam = Baglam::olustur(cli)?;
    let mut gunluk = baglam.gunluk()?;
    let sira = gunluk
        .acik_oturum()
        .map(|o| o.sira)
        .ok_or(Hata::AcikOturumYok)?;
    let simdi = arg.simdi.unwrap_or(baglam.simdi);
    let oturum = gunluk.getir_mut(sira)?;
    let gecen_i64 = (simdi - oturum.baslangic) / 60;
    let gecen = i32::try_from(gecen_i64).unwrap_or(i32::MAX);
    oturum.kesinti_ekle(&arg.kaynak, arg.dakika, gecen)?;
    let guncel = gunluk.getir(sira)?.clone();
    baglam.depo.oturum_ekle(&guncel)?;
    Ok(format!(
        "kesinti kaydedildi: oturum #{sira}  kaynak={}  {}  toplam={} (gecen {})",
        arg.kaynak,
        sure_yaz(arg.dakika),
        sure_yaz(guncel.toplam_kesinti()),
        sure_yaz(gecen)
    ))
}

fn bagimlilik_calistir(cli: &Cli, komut: &BagimlilikKomut) -> Sonuc<String> {
    let baglam = Baglam::olustur(cli)?;
    let mut depo = baglam.depo.gorevleri_oku()?;
    match komut {
        BagimlilikKomut::Ekle(arg) => {
            depo.bagimlilik_ekle(arg.gorev, arg.bagimlilik)?;
            baglam.depo.gorevleri_yaz(&depo, baglam.simdi)?;
            Ok(format!(
                "bagimlilik eklendi: #{} -> #{}",
                arg.gorev, arg.bagimlilik
            ))
        }
        BagimlilikKomut::Kaldir(arg) => {
            depo.bagimlilik_kaldir(arg.gorev, arg.bagimlilik)?;
            baglam.depo.gorevleri_yaz(&depo, baglam.simdi)?;
            Ok(format!(
                "bagimlilik kaldirildi: #{} -> #{}",
                arg.gorev, arg.bagimlilik
            ))
        }
        BagimlilikKomut::Agac(arg) => {
            let katmanlar = depo.bagimlilik_agaci(arg.gorev)?;
            if katmanlar.is_empty() {
                return Ok(format!("#{} bagimliligi yok", arg.gorev));
            }
            let mut satirlar = vec![format!("#{} {}", arg.gorev, depo.getir(arg.gorev)?.baslik)];
            for (derinlik, katman) in katmanlar.iter().enumerate() {
                for id in katman {
                    let hedef = depo.getir(*id)?;
                    satirlar.push(format!(
                        "{}-> #{id} {} [{}]",
                        "  ".repeat(derinlik + 1),
                        hedef.baslik,
                        hedef.durum.ad()
                    ));
                }
            }
            Ok(satirlar.join("\n"))
        }
    }
}

fn dongu_calistir(cli: &Cli, arg: &DonguArg) -> Sonuc<String> {
    let baglam = Baglam::olustur(cli)?;
    let mut depo = baglam.depo.gorevleri_oku()?;
    let mut gunluk = baglam.gunluk()?;
    if let Some(acik) = gunluk.acik_oturum() {
        return Err(Hata::OturumCakisiyor {
            acik_sira: acik.sira,
        });
    }
    depo.getir(arg.id)?;
    depo.baslatilabilir(arg.id)?;
    let mola_turu: OturumTuru = arg.mola_tipi.into();

    let mut satirlar: Vec<String> = Vec::new();
    let mut simdi = baglam.simdi;
    // Ozet yalnizca bu calistirmada uretilen oturumlari kapsar; onceki oturumlar
    // haftalik raporda zaten var.
    let ozet_baslangic = gunluk.uzunluk();
    depo.durum_degistir(arg.id, Durum::Calisiyor, simdi)?;
    let mut odak_sayisi: u32 = 0;
    let mut son_odak: Option<Oturum> = None;
    for tur_no in 1..=arg.dongu {
        let odak_sira = gunluk.baslat(Some(arg.id), OturumTuru::Odak, arg.odak_dakika, simdi)?;
        if let Some(kesinti) = arg.kesinti_dakika {
            if kesinti > 0 {
                gunluk.getir_mut(odak_sira)?.kesinti_ekle(
                    &arg.kesinti_kaynak,
                    kesinti,
                    arg.odak_dakika,
                )?;
            }
        }
        let odak_bit = simdi + i64::from(arg.odak_dakika) * 60;
        gunluk.getir_mut(odak_sira)?.bitir(odak_bit)?;
        if !arg.kuru {
            bekle(i64::from(arg.odak_dakika) * 60);
        }
        let odak = gunluk.getir(odak_sira)?.clone();
        baglam.depo.oturum_ekle(&odak)?;
        satirlar.push(format!(
            "dongu {}/{} odak: {}  {}",
            tur_no,
            arg.dongu,
            odak.ozet()?,
            sure_yaz(odak.odak_dakika()?)
        ));
        odak_sayisi += 1;
        son_odak = Some(odak);
        simdi = odak_bit;

        if arg.mola_dakika > 0 {
            let mola_sira = gunluk.baslat(None, mola_turu, arg.mola_dakika, simdi)?;
            let mola_bit = simdi + i64::from(arg.mola_dakika) * 60;
            if !arg.kuru {
                bekle(i64::from(arg.mola_dakika) * 60);
            }
            gunluk.getir_mut(mola_sira)?.bitir(mola_bit)?;
            let mola = gunluk.getir(mola_sira)?.clone();
            baglam.depo.oturum_ekle(&mola)?;
            satirlar.push(format!(
                "dongu {}/{} mola: {}  {}",
                tur_no,
                arg.dongu,
                mola.tur.ad(),
                sure_yaz(arg.mola_dakika)
            ));
            simdi = mola_bit;
        }
    }
    if arg.dongu == 1 {
        // Tek tam dongude gorev tamamlanir; coklu donguda kullanici `stop` ile kapatir.
        depo.durum_degistir(arg.id, Durum::Tamamlandi, simdi)?;
    }
    baglam.depo.gorevleri_yaz(&depo, simdi)?;
    if let Some(odak) = &son_odak {
        gunluk_satir_ekle(&baglam, odak)?;
    }
    let odak_toplam = gunluk
        .oturumlar()
        .iter()
        .skip(ozet_baslangic)
        .filter(|o| o.tur.odak_mi())
        .try_fold(0i32, |toplam, o| Ok::<i32, Hata>(toplam + o.odak_dakika()?))?;
    satirlar.push(format!(
        "toplam: {odak_sayisi} odak oturumu, {} odak suresi  gorev=#{} durum={}",
        sure_yaz(odak_toplam),
        arg.id,
        depo.getir(arg.id)?.durum.ad()
    ));
    Ok(satirlar.join("\n"))
}

fn plan_uret(cli: &Cli, arg: &PlanArg) -> Sonuc<String> {
    let baglam = Baglam::olustur(cli)?;
    let depo = baglam.depo.gorevleri_oku()?;
    let tarih = match &arg.tarih {
        Some(metin) => {
            tarih_ayir(metin)?;
            metin.clone()
        }
        None => baglam.bugun(),
    };
    let kapasite = GunlukKapasite::yeni(&tarih, arg.kapasite, arg.sabit)?;
    Ok(planla(&depo, &kapasite)?.metin())
}

fn rapor_uret(cli: &Cli, arg: &RaporArg) -> Sonuc<String> {
    let saat_dilimi = SaatDilimi::yeni(cli.tz_offset)?;
    if !matches!(arg.format.as_str(), "json" | "md" | "ikisi") {
        return Err(Hata::GecersizFormat {
            girdi: arg.format.clone(),
        });
    }
    let simdi = simdi_epoch();
    let gun_no = saat_dilimi.gun_no(simdi);
    let kok = arg
        .cikti
        .clone()
        .or_else(|| cli.veri.clone())
        .unwrap_or_else(|| PathBuf::from("veri"));
    let depo = Depo::yeni(kok);
    depo.hazirla()?;
    let pazartesi = match &arg.hafta {
        Some(hafta) => hafta_ayir(hafta)?,
        None => gun_no - (gun_no + 3).rem_euclid(7),
    };
    let gunluk = depo.oturumlari_oku().map(OturumGunlugu::yukle)?;
    let gorev_depo = depo.gorevleri_oku()?;
    let rapor = rapor_olustur(pazartesi, GUN_SANIYE, &gunluk, &gorev_depo)?;

    let mut satirlar = vec![
        rapor.ozet_satiri(),
        format!(
            "gerceklesme: %{}  (tahmin {} - gerceklesen {})",
            rapor.gerceklesme_yuzdesi(),
            sure_yaz(rapor.toplam_tahmini_dakika),
            sure_yaz(rapor.toplam_odak_dakika)
        ),
    ];
    if arg.format == "json" || arg.format == "ikisi" {
        let ad = rapor.dosya_adi("json");
        depo.rapor_yaz(&ad, &rapor.json()?)?;
        satirlar.push(format!("yazildi: {}", depo.rapor_yolu(&ad).display()));
    }
    if arg.format == "md" || arg.format == "ikisi" {
        let ad = rapor.dosya_adi("md");
        depo.rapor_yaz(&ad, &rapor.markdown())?;
        satirlar.push(format!("yazildi: {}", depo.rapor_yolu(&ad).display()));
    }
    Ok(satirlar.join("\n"))
}

/// Oturum kapandiginda gunluk Markdown dosyasina satir ekler.
fn gunluk_satir_ekle(baglam: &Baglam, oturum: &Oturum) -> Sonuc<()> {
    let tarih = tarih_yaz(baglam.saat_dilimi.gun_no(oturum.baslangic));
    let mut icerik = baglam.depo.gunluk_oku(&tarih)?;
    if icerik.is_empty() {
        icerik.push_str(&format!("# FocusCompass gunlugu - {tarih}\n\n"));
    } else if !icerik.ends_with('\n') {
        icerik.push('\n');
    }
    icerik.push_str(&format!(
        "- {} {}  {}\n",
        iso8601_utc(oturum.baslangic),
        oturum.tur.ad(),
        oturum.ozet()?
    ));
    baglam.depo.gunluk_yaz(&tarih, &icerik)
}

/// Gercekten bekler; yalnizca `--kuru` verilmediginde cagrilir.
fn bekle(saniye: i64) {
    std::thread::sleep(std::time::Duration::from_secs(saniye as u64));
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use std::path::Path;

    fn gecici_kok(etiket: &str) -> PathBuf {
        let kok = std::env::temp_dir().join(format!("fc-cli-{etiket}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&kok);
        std::fs::create_dir_all(&kok).unwrap();
        kok
    }

    fn cli(kok: &Path, komut: Komut) -> Cli {
        Cli {
            veri: Some(kok.to_path_buf()),
            tz_offset: 180,
            komut,
        }
    }

    fn ekle_komutu(baslik: &str, dakika: i32, oncelik: &str) -> Komut {
        Komut::Add(EkleArg {
            basliklar: vec![baslik.to_string()],
            dakika,
            oncelik: oncelik.to_string(),
            etiket: vec!["rapor".to_string()],
            bagimlilik: Vec::new(),
        })
    }

    fn start_komutu(id: u32) -> Komut {
        Komut::Start(BaslatArg {
            id,
            dakika: 25,
            tur: TurSecimi::Odak,
        })
    }

    #[test]
    fn add_list_akisi_calisir() {
        let kok = gecici_kok("add-list");
        let cikti = calistir(&cli(&kok, ekle_komutu("Rapor taslagi", 45, "yuksek"))).unwrap();
        assert!(cikti.starts_with("eklendi #1"), "{cikti}");
        assert!(cikti.contains("yuksek"), "{cikti}");
        assert!(cikti.contains("etiket=rapor"), "{cikti}");
        let cikti = calistir(&cli(
            &kok,
            Komut::List(ListeArg {
                durum: None,
                etiket: None,
                ozet: false,
            }),
        ))
        .unwrap();
        assert!(cikti.contains("Rapor taslagi"), "{cikti}");
        assert!(cikti.contains("1 gorev"), "{cikti}");
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn negatif_sure_ile_ekleme_reddedilir() {
        let kok = gecici_kok("negatif");
        let sonuc = calistir(&cli(&kok, ekle_komutu("Hatali", -5, "normal")));
        assert!(matches!(sonuc, Err(Hata::NegatifSure { deger: -5 })));
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn gecersiz_oncelik_reddedilir() {
        let kok = gecici_kok("oncelik");
        let sonuc = calistir(&cli(&kok, ekle_komutu("Hatali", 10, "cok")));
        assert!(matches!(sonuc, Err(Hata::GecersizOncelik { .. })));
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn ust_uste_oturum_reddedilir() {
        let kok = gecici_kok("cakisma");
        calistir(&cli(&kok, ekle_komutu("Yaz", 25, "normal"))).unwrap();
        calistir(&cli(&kok, start_komutu(1))).unwrap();
        let sonuc = calistir(&cli(&kok, start_komutu(1)));
        assert!(matches!(sonuc, Err(Hata::OturumCakisiyor { acik_sira: 1 })));
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn start_stop_akisi_gunluk_dosyasi_yazar() {
        let kok = gecici_kok("start-stop");
        calistir(&cli(&kok, ekle_komutu("Yaz", 25, "normal"))).unwrap();
        let cikti = calistir(&cli(&kok, start_komutu(1))).unwrap();
        assert!(cikti.contains("oturum #1 basladi"), "{cikti}");
        let depo = Depo::yeni(&kok);
        let oturumlar = depo.oturumlari_oku().unwrap();
        assert_eq!(oturumlar.len(), 1);
        assert!(oturumlar[0].bitis.is_none());

        let bitis = oturumlar[0].baslangic + 25 * 60;
        let cikti = calistir(&cli(
            &kok,
            Komut::Stop(DurdurArg {
                ertele: false,
                bitis: Some(bitis),
            }),
        ))
        .unwrap();
        assert!(cikti.contains("oturum #1 kapandi"), "{cikti}");
        assert!(cikti.contains("durum=tamamlandi"), "{cikti}");

        let gunluk = depo.gunluk_oku(&tarih_yaz(
            SaatDilimi::yeni(180)
                .unwrap()
                .gun_no(oturumlar[0].baslangic),
        ));
        assert!(gunluk.unwrap().contains("odak"));
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn stop_erteleme_bayragi_durumu_degistirir() {
        let kok = gecici_kok("stop-ertele");
        calistir(&cli(&kok, ekle_komutu("Yaz", 25, "normal"))).unwrap();
        calistir(&cli(&kok, start_komutu(1))).unwrap();
        let cikti = calistir(&cli(
            &kok,
            Komut::Stop(DurdurArg {
                ertele: true,
                bitis: None,
            }),
        ))
        .unwrap();
        assert!(cikti.contains("durum=ertelendi"), "{cikti}");
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn acik_oturum_yoksa_stop_hata_verir() {
        let kok = gecici_kok("stop-bos");
        calistir(&cli(&kok, ekle_komutu("Yaz", 25, "normal"))).unwrap();
        let sonuc = calistir(&cli(
            &kok,
            Komut::Stop(DurdurArg {
                ertele: false,
                bitis: None,
            }),
        ));
        assert!(matches!(sonuc, Err(Hata::AcikOturumYok)));
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn start_bagimlilik_tamamlanmadiysa_reddedilir() {
        let kok = gecici_kok("start-engel");
        calistir(&cli(&kok, ekle_komutu("A", 10, "normal"))).unwrap();
        calistir(&cli(&kok, ekle_komutu("B", 10, "normal"))).unwrap();
        calistir(&cli(
            &kok,
            Komut::Dep(BagimlilikKomut::Ekle(BagimlilikCiftArg {
                gorev: 2,
                bagimlilik: 1,
            })),
        ))
        .unwrap();
        let sonuc = calistir(&cli(&kok, start_komutu(2)));
        assert!(matches!(sonuc, Err(Hata::BaslatilamazSebep { .. })));
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn plan_kapasite_ile_calisir() {
        let kok = gecici_kok("plan");
        calistir(&cli(&kok, ekle_komutu("Kucuk", 30, "yuksek"))).unwrap();
        calistir(&cli(&kok, ekle_komutu("Buyuk", 300, "dusuk"))).unwrap();
        let cikti = calistir(&cli(
            &kok,
            Komut::Plan(PlanArg {
                kapasite: 120,
                sabit: 60,
                tarih: Some("2026-09-29".to_string()),
            }),
        ))
        .unwrap();
        assert!(cikti.contains("2026-09-29"), "{cikti}");
        assert!(cikti.contains("kullanilabilir 60 dk"), "{cikti}");
        assert!(cikti.contains("BUGUN SIGANLAR"), "{cikti}");
        assert!(cikti.contains("BUGUN SIGMAYANLAR"), "{cikti}");
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn plan_gecersiz_tarih_reddedilir() {
        let kok = gecici_kok("plan-tarih");
        let sonuc = calistir(&cli(
            &kok,
            Komut::Plan(PlanArg {
                kapasite: 120,
                sabit: 0,
                tarih: Some("29-09-2026".to_string()),
            }),
        ));
        assert!(matches!(sonuc, Err(Hata::GecersizTarih { .. })));
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn report_iki_dosya_yazar() {
        let kok = gecici_kok("rapor");
        calistir(&cli(&kok, ekle_komutu("Yaz", 25, "normal"))).unwrap();
        let cikti = calistir(&cli(
            &kok,
            Komut::Report(RaporArg {
                hafta: Some("2026-W40".to_string()),
                format: "ikisi".to_string(),
                cikti: Some(kok.clone()),
            }),
        ))
        .unwrap();
        assert!(cikti.contains("2026-W40"), "{cikti}");
        assert!(kok.join("raporlar").join("hafta-2026-W40.json").exists());
        assert!(kok.join("raporlar").join("hafta-2026-W40.md").exists());
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn report_gecersiz_format_reddedilir() {
        let kok = gecici_kok("rapor-format");
        let sonuc = calistir(&cli(
            &kok,
            Komut::Report(RaporArg {
                hafta: Some("2026-W40".to_string()),
                format: "html".to_string(),
                cikti: Some(kok.clone()),
            }),
        ));
        assert!(matches!(sonuc, Err(Hata::GecersizFormat { .. })));
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn report_gecersiz_hafta_reddedilir() {
        let kok = gecici_kok("rapor-hafta");
        let sonuc = calistir(&cli(
            &kok,
            Komut::Report(RaporArg {
                hafta: Some("2026-40".to_string()),
                format: "md".to_string(),
                cikti: Some(kok.clone()),
            }),
        ));
        assert!(matches!(sonuc, Err(Hata::GecersizHafta { .. })));
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn dep_ekle_dongu_yaratmaz() {
        let kok = gecici_kok("dep");
        calistir(&cli(&kok, ekle_komutu("A", 10, "normal"))).unwrap();
        calistir(&cli(&kok, ekle_komutu("B", 10, "normal"))).unwrap();
        let cikti = calistir(&cli(
            &kok,
            Komut::Dep(BagimlilikKomut::Ekle(BagimlilikCiftArg {
                gorev: 2,
                bagimlilik: 1,
            })),
        ))
        .unwrap();
        assert!(cikti.contains("bagimlilik eklendi"), "{cikti}");
        let sonuc = calistir(&cli(
            &kok,
            Komut::Dep(BagimlilikKomut::Ekle(BagimlilikCiftArg {
                gorev: 1,
                bagimlilik: 2,
            })),
        ));
        assert!(matches!(sonuc, Err(Hata::BagimlilikDongusu { .. })));
        // Dosyada dongu kalmadi; yeniden yukleme temiz.
        let depo = Depo::yeni(&kok).gorevleri_oku().unwrap();
        assert!(depo.dongu_ara().is_none());
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn dep_agac_bagimliliklari_gosterir() {
        let kok = gecici_kok("agac");
        calistir(&cli(&kok, ekle_komutu("A", 10, "normal"))).unwrap();
        calistir(&cli(&kok, ekle_komutu("B", 10, "normal"))).unwrap();
        calistir(&cli(
            &kok,
            Komut::Dep(BagimlilikKomut::Ekle(BagimlilikCiftArg {
                gorev: 2,
                bagimlilik: 1,
            })),
        ))
        .unwrap();
        let cikti = calistir(&cli(
            &kok,
            Komut::Dep(BagimlilikKomut::Agac(AgacArg { gorev: 2 })),
        ))
        .unwrap();
        assert!(cikti.contains("#2 B"), "{cikti}");
        assert!(cikti.contains("-> #1 A"), "{cikti}");
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn cycle_kuru_kayit_uretir() {
        let kok = gecici_kok("cycle");
        calistir(&cli(&kok, ekle_komutu("Yaz", 50, "normal"))).unwrap();
        let cikti = calistir(&cli(
            &kok,
            Komut::Cycle(DonguArg {
                id: 1,
                odak_dakika: 25,
                mola_dakika: 5,
                dongu: 2,
                mola_tipi: TurSecimi::Uzun,
                kesinti_dakika: Some(3),
                kesinti_kaynak: "toplanti".to_string(),
                kuru: true,
            }),
        ))
        .unwrap();
        assert!(cikti.contains("dongu 1/2 odak"), "{cikti}");
        assert!(cikti.contains("dongu 2/2 odak"), "{cikti}");
        let oturumlar = Depo::yeni(&kok).oturumlari_oku().unwrap();
        assert_eq!(oturumlar.len(), 4, "2 odak + 2 mola");
        assert_eq!(oturumlar[0].toplam_kesinti(), 3);
        assert!(cikti.contains("durum=calisiyor"), "{cikti}");
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn cycle_tek_dongu_gorevi_tamamlar() {
        let kok = gecici_kok("cycle-tek");
        calistir(&cli(&kok, ekle_komutu("Yaz", 50, "normal"))).unwrap();
        let cikti = calistir(&cli(
            &kok,
            Komut::Cycle(DonguArg {
                id: 1,
                odak_dakika: 25,
                mola_dakika: 0,
                dongu: 1,
                mola_tipi: TurSecimi::Kisa,
                kesinti_dakika: None,
                kesinti_kaynak: "toplanti".to_string(),
                kuru: true,
            }),
        ))
        .unwrap();
        assert!(cikti.contains("durum=tamamlandi"), "{cikti}");
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn cycle_engelli_gorevi_baslatmaz() {
        let kok = gecici_kok("cycle-engel");
        calistir(&cli(&kok, ekle_komutu("A", 10, "normal"))).unwrap();
        calistir(&cli(&kok, ekle_komutu("B", 10, "normal"))).unwrap();
        calistir(&cli(
            &kok,
            Komut::Dep(BagimlilikKomut::Ekle(BagimlilikCiftArg {
                gorev: 2,
                bagimlilik: 1,
            })),
        ))
        .unwrap();
        let sonuc = calistir(&cli(
            &kok,
            Komut::Cycle(DonguArg {
                id: 2,
                odak_dakika: 25,
                mola_dakika: 5,
                dongu: 1,
                mola_tipi: TurSecimi::Kisa,
                kesinti_dakika: None,
                kesinti_kaynak: "toplanti".to_string(),
                kuru: true,
            }),
        ));
        assert!(matches!(sonuc, Err(Hata::BaslatilamazSebep { .. })));
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn interrupt_acik_oturuma_kayit_yapar() {
        let kok = gecici_kok("interrupt");
        calistir(&cli(&kok, ekle_komutu("Yaz", 25, "normal"))).unwrap();
        let baslangic = simdi_epoch();
        calistir(&cli(&kok, start_komutu(1))).unwrap();
        let cikti = calistir(&cli(
            &kok,
            Komut::Interrupt(KesintiArg {
                kaynak: "toplanti".to_string(),
                dakika: 5,
                simdi: Some(baslangic + 600),
            }),
        ))
        .unwrap();
        assert!(cikti.contains("kesinti kaydedildi"), "{cikti}");
        let gunluk = OturumGunlugu::yukle(Depo::yeni(&kok).oturumlari_oku().unwrap());
        assert_eq!(gunluk.getir(1).unwrap().toplam_kesinti(), 5);
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn interrupt_acik_oturum_yoksa_hata_verir() {
        let kok = gecici_kok("interrupt-yok");
        let sonuc = calistir(&cli(
            &kok,
            Komut::Interrupt(KesintiArg {
                kaynak: "toplanti".to_string(),
                dakika: 5,
                simdi: None,
            }),
        ));
        assert!(matches!(sonuc, Err(Hata::AcikOturumYok)));
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn list_durum_ve_etiket_filtresi_calisir() {
        let kok = gecici_kok("list-filtre");
        calistir(&cli(&kok, ekle_komutu("A", 10, "normal"))).unwrap();
        calistir(&cli(&kok, ekle_komutu("B", 10, "normal"))).unwrap();
        let cikti = calistir(&cli(
            &kok,
            Komut::List(ListeArg {
                durum: Some("ertelendi".to_string()),
                etiket: None,
                ozet: false,
            }),
        ))
        .unwrap();
        assert_eq!(cikti, "kayitli gorev yok");
        let cikti = calistir(&cli(
            &kok,
            Komut::List(ListeArg {
                durum: Some("bekliyor".to_string()),
                etiket: Some("Rapor".to_string()),
                ozet: true,
            }),
        ))
        .unwrap();
        assert!(cikti.contains("2 gorev"), "{cikti}");
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn gecersiz_saat_dilimi_reddedilir() {
        let kok = gecici_kok("tz");
        let bozuk = Cli {
            veri: Some(kok.clone()),
            tz_offset: 900,
            komut: Komut::List(ListeArg {
                durum: None,
                etiket: None,
                ozet: true,
            }),
        };
        assert!(matches!(
            calistir(&bozuk),
            Err(Hata::GecersizSaatDilimi { ofset_dakika: 900 })
        ));
        let _ = std::fs::remove_dir_all(&kok);
    }

    #[test]
    fn tur_secimi_donusumu_dogrudur() {
        assert_eq!(OturumTuru::from(TurSecimi::Kisa), OturumTuru::KisaMola);
        assert_eq!(OturumTuru::from(TurSecimi::Uzun), OturumTuru::UzunMola);
        assert_eq!(OturumTuru::from(TurSecimi::Odak), OturumTuru::Odak);
    }
}
