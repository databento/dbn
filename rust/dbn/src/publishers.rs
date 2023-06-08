use num_enum::{IntoPrimitive, TryFromPrimitive};

/// A trading execution venue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Venue {
    /// CME GLOBEX
    Glbx = 1,
    /// NASDAQ
    Xnas = 2,
    /// NASDAQ OMX BX
    Xbos = 3,
    /// NASDAQ OMX PSX
    Xpsx = 4,
    /// CBOE BZX U.S. EQUITIES EXCHANGE
    Bats = 5,
    /// CBOE BYX U.S. EQUITIES EXCHANGE
    Baty = 6,
    /// CBOE EDGA U.S. EQUITIES EXCHANGE
    Edga = 7,
    /// CBOE EDGX U.S. EQUITIES EXCHANGE
    Edgx = 8,
    /// New York Stock Exchange
    Xnys = 9,
    /// NYSE NATIONAL, INC.
    Xcis = 10,
    /// NYSE AMERICAN
    Xase = 11,
    /// NYSE ARCA
    Arcx = 12,
    /// NYSE CHICAGO, INC.
    Xchi = 13,
    /// INVESTORS EXCHANGE
    Iexg = 14,
    /// FINRA/NASDAQ TRF CARTERET
    Finn = 15,
    /// FINRA/NASDAQ TRF CHICAGO
    Finc = 16,
    /// FINRA/NYSE TRF
    Finy = 17,
    /// MEMX LLC EQUITIES
    Memx = 18,
    /// MIAX PEARL EQUITIES
    Eprl = 19,
    /// NYSE AMERICAN OPTIONS
    Amxo = 20,
    /// BOX OPTIONS EXCHANGE
    Xbox = 21,
    /// CBOE OPTIONS EXCHANGE
    Xcbo = 22,
    /// MIAX EMERALD
    Emld = 23,
    /// Cboe EDGX Options Exchange
    Edgo = 24,
    /// NASDAQ GEMX
    Gmni = 25,
    /// NASDAQ ISE
    Xisx = 26,
    /// NASDAQ MRX
    Mcry = 27,
    /// MIAX INTERNATIONAL SECURITIES
    Xmio = 28,
    /// NYSE ARCA OPTIONS
    Arco = 29,
    /// OPRA
    Opra = 30,
    /// MIAX PEARL
    Mprl = 31,
    /// NASDAQ OPTIONS MARKET
    Xndq = 32,
    /// NASDAQ BX OPTIONS
    Xbxo = 33,
    /// CBOE C2 OPTIONS EXCHANGE
    C2Ox = 34,
    /// NASDAQ PHLX
    Xphl = 35,
    /// CBOE BZX Options Exchange
    Bato = 36,
}

/// The number of Venue variants.
pub const VENUE_COUNT: usize = 36;

impl Venue {
    /// Convert a Venue to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Venue::Glbx => "GLBX",
            Venue::Xnas => "XNAS",
            Venue::Xbos => "XBOS",
            Venue::Xpsx => "XPSX",
            Venue::Bats => "BATS",
            Venue::Baty => "BATY",
            Venue::Edga => "EDGA",
            Venue::Edgx => "EDGX",
            Venue::Xnys => "XNYS",
            Venue::Xcis => "XCIS",
            Venue::Xase => "XASE",
            Venue::Arcx => "ARCX",
            Venue::Xchi => "XCHI",
            Venue::Iexg => "IEXG",
            Venue::Finn => "FINN",
            Venue::Finc => "FINC",
            Venue::Finy => "FINY",
            Venue::Memx => "MEMX",
            Venue::Eprl => "EPRL",
            Venue::Amxo => "AMXO",
            Venue::Xbox => "XBOX",
            Venue::Xcbo => "XCBO",
            Venue::Emld => "EMLD",
            Venue::Edgo => "EDGO",
            Venue::Gmni => "GMNI",
            Venue::Xisx => "XISX",
            Venue::Mcry => "MCRY",
            Venue::Xmio => "XMIO",
            Venue::Arco => "ARCO",
            Venue::Opra => "OPRA",
            Venue::Mprl => "MPRL",
            Venue::Xndq => "XNDQ",
            Venue::Xbxo => "XBXO",
            Venue::C2Ox => "C2OX",
            Venue::Xphl => "XPHL",
            Venue::Bato => "BATO",
        }
    }
}

impl AsRef<str> for Venue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::str::FromStr for Venue {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        match s {
            "GLBX" => Ok(Venue::Glbx),
            "XNAS" => Ok(Venue::Xnas),
            "XBOS" => Ok(Venue::Xbos),
            "XPSX" => Ok(Venue::Xpsx),
            "BATS" => Ok(Venue::Bats),
            "BATY" => Ok(Venue::Baty),
            "EDGA" => Ok(Venue::Edga),
            "EDGX" => Ok(Venue::Edgx),
            "XNYS" => Ok(Venue::Xnys),
            "XCIS" => Ok(Venue::Xcis),
            "XASE" => Ok(Venue::Xase),
            "ARCX" => Ok(Venue::Arcx),
            "XCHI" => Ok(Venue::Xchi),
            "IEXG" => Ok(Venue::Iexg),
            "FINN" => Ok(Venue::Finn),
            "FINC" => Ok(Venue::Finc),
            "FINY" => Ok(Venue::Finy),
            "MEMX" => Ok(Venue::Memx),
            "EPRL" => Ok(Venue::Eprl),
            "AMXO" => Ok(Venue::Amxo),
            "XBOX" => Ok(Venue::Xbox),
            "XCBO" => Ok(Venue::Xcbo),
            "EMLD" => Ok(Venue::Emld),
            "EDGO" => Ok(Venue::Edgo),
            "GMNI" => Ok(Venue::Gmni),
            "XISX" => Ok(Venue::Xisx),
            "MCRY" => Ok(Venue::Mcry),
            "XMIO" => Ok(Venue::Xmio),
            "ARCO" => Ok(Venue::Arco),
            "OPRA" => Ok(Venue::Opra),
            "MPRL" => Ok(Venue::Mprl),
            "XNDQ" => Ok(Venue::Xndq),
            "XBXO" => Ok(Venue::Xbxo),
            "C2OX" => Ok(Venue::C2Ox),
            "XPHL" => Ok(Venue::Xphl),
            "BATO" => Ok(Venue::Bato),
            _ => Err(anyhow::format_err!("String doesn't match any valid Venue")),
        }
    }
}

/// A source of data.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Dataset {
    /// CME MDP 3.0 Market Data
    GlbxMdp3 = 1,
    /// Nasdaq XNAS TotalView-ITCH
    XnasItch = 2,
    /// Nasdaq XBOS TotalView-ITCH
    XbosItch = 3,
    /// Nasdaq XPSX TotalView-ITCH
    XpsxItch = 4,
    /// CBOE BZX
    BatsPitch = 5,
    /// CBOE BYX
    BatyPitch = 6,
    /// CBOE EDGA
    EdgaPitch = 7,
    /// CBOE EDGX
    EdgxPitch = 8,
    /// NYSE
    XnysPillar = 9,
    /// NYSE National
    XcisPillar = 10,
    /// NYSE American
    XasePillar = 11,
    /// NYSE Chicago
    XchiPillar = 12,
    /// NYSE National BBO
    XcisBbo = 13,
    /// NYSE National TRADES
    XcisTrades = 14,
    /// MEMX Memoir Depth
    MemxMemoir = 15,
    /// MIAX Pearl Depth
    EprlDom = 16,
    /// Finra/Nasdaq TRF
    FinnNls = 17,
    /// Finra/NYSE TRF
    FinyTrades = 18,
    /// OPRA Binary Recipient
    OpraPillar = 19,
    /// Databento Equities Basic
    DbeqBasic = 20,
}

/// The number of Dataset variants.
pub const DATASET_COUNT: usize = 20;

impl Dataset {
    /// Convert a Dataset to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Dataset::GlbxMdp3 => "GLBX.MDP3",
            Dataset::XnasItch => "XNAS.ITCH",
            Dataset::XbosItch => "XBOS.ITCH",
            Dataset::XpsxItch => "XPSX.ITCH",
            Dataset::BatsPitch => "BATS.PITCH",
            Dataset::BatyPitch => "BATY.PITCH",
            Dataset::EdgaPitch => "EDGA.PITCH",
            Dataset::EdgxPitch => "EDGX.PITCH",
            Dataset::XnysPillar => "XNYS.PILLAR",
            Dataset::XcisPillar => "XCIS.PILLAR",
            Dataset::XasePillar => "XASE.PILLAR",
            Dataset::XchiPillar => "XCHI.PILLAR",
            Dataset::XcisBbo => "XCIS.BBO",
            Dataset::XcisTrades => "XCIS.TRADES",
            Dataset::MemxMemoir => "MEMX.MEMOIR",
            Dataset::EprlDom => "EPRL.DOM",
            Dataset::FinnNls => "FINN.NLS",
            Dataset::FinyTrades => "FINY.TRADES",
            Dataset::OpraPillar => "OPRA.PILLAR",
            Dataset::DbeqBasic => "DBEQ.BASIC",
        }
    }
}

impl AsRef<str> for Dataset {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::str::FromStr for Dataset {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        match s {
            "GLBX.MDP3" => Ok(Dataset::GlbxMdp3),
            "XNAS.ITCH" => Ok(Dataset::XnasItch),
            "XBOS.ITCH" => Ok(Dataset::XbosItch),
            "XPSX.ITCH" => Ok(Dataset::XpsxItch),
            "BATS.PITCH" => Ok(Dataset::BatsPitch),
            "BATY.PITCH" => Ok(Dataset::BatyPitch),
            "EDGA.PITCH" => Ok(Dataset::EdgaPitch),
            "EDGX.PITCH" => Ok(Dataset::EdgxPitch),
            "XNYS.PILLAR" => Ok(Dataset::XnysPillar),
            "XCIS.PILLAR" => Ok(Dataset::XcisPillar),
            "XASE.PILLAR" => Ok(Dataset::XasePillar),
            "XCHI.PILLAR" => Ok(Dataset::XchiPillar),
            "XCIS.BBO" => Ok(Dataset::XcisBbo),
            "XCIS.TRADES" => Ok(Dataset::XcisTrades),
            "MEMX.MEMOIR" => Ok(Dataset::MemxMemoir),
            "EPRL.DOM" => Ok(Dataset::EprlDom),
            "FINN.NLS" => Ok(Dataset::FinnNls),
            "FINY.TRADES" => Ok(Dataset::FinyTrades),
            "OPRA.PILLAR" => Ok(Dataset::OpraPillar),
            "DBEQ.BASIC" => Ok(Dataset::DbeqBasic),
            _ => Err(anyhow::format_err!(
                "String doesn't match any valid Dataset"
            )),
        }
    }
}

/// A specific Venue from a specific data source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Publisher {
    /// CME Globex MDP 3.0
    GlbxMdp3Glbx = 1,
    /// Nasdaq TotalView ITCH
    XnasItchXnas = 2,
    /// Nasdaq XBOS TotalView ITCH
    XbosItchXbos = 3,
    /// Nasdaq XPSX TotalView ITCH
    XpsxItchXpsx = 4,
    /// CBOE BZX
    BatsPitchBats = 5,
    /// CBOE BYX
    BatyPitchBats = 6,
    /// CBOE EDGA
    EdgaPitchEdga = 7,
    /// CBOE EDGX
    EdgxPitchEdgx = 8,
    /// NYSE
    XnysPillarXnys = 9,
    /// NYSE National
    XcisPillarXcis = 10,
    /// NYSE American
    XasePillarXase = 11,
    /// NYSE Chicago
    XchiPillarXchi = 12,
    /// NYSE National BBO
    XcisBboXcis = 13,
    /// NYSE National Trades
    XcisTradesXcis = 14,
    /// MEMX Memoir Depth
    MemxMemoirMemx = 15,
    /// MIAX Pearl Depth
    EprlDomEprl = 16,
    /// FINRA/NASDAQ TRF CARTERET
    FinnNlsFinn = 17,
    /// FINRA/NASDAQ TRF CHICAGO
    FinnNlsFinc = 18,
    /// FINRA/NYSE TRF
    FinyTradesFiny = 19,
    /// OPRA - NYSE AMERICAN OPTIONS
    OpraPillarAmxo = 20,
    /// OPRA - BOX OPTIONS EXCHANGE
    OpraPillarXbox = 21,
    /// OPRA - CBOE OPTIONS EXCHANGE
    OpraPillarXcbo = 22,
    /// OPRA - MIAX EMERALD
    OpraPillarEmld = 23,
    /// OPRA - Cboe EDGX Options Exchange
    OpraPillarEdgo = 24,
    /// OPRA - NASDAQ GEMX
    OpraPillarGmni = 25,
    /// OPRA - NASDAQ ISE
    OpraPillarXisx = 26,
    /// OPRA - NASDAQ MRX
    OpraPillarMcry = 27,
    /// OPRA - MIAX INTERNATIONAL SECURITIES
    OpraPillarXmio = 28,
    /// OPRA - NYSE ARCA OPTIONS
    OpraPillarArco = 29,
    /// OPRA - OPRA
    OpraPillarOpra = 30,
    /// OPRA - MIAX PEARL
    OpraPillarMprl = 31,
    /// OPRA - NASDAQ OPTIONS MARKET
    OpraPillarXndq = 32,
    /// OPRA - NASDAQ BX OPTIONS
    OpraPillarXbxo = 33,
    /// OPRA - CBOE C2 OPTIONS EXCHANGE
    OpraPillarC2Ox = 34,
    /// OPRA - NASDAQ PHLX
    OpraPillarXphl = 35,
    /// OPRA - CBOE BZX Options Exchange
    OpraPillarBato = 36,
}

/// The number of Publisher variants.
pub const PUBLISHER_COUNT: usize = 36;

impl Publisher {
    /// Convert a Publisher to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Publisher::GlbxMdp3Glbx => "GLBX.MDP3.GLBX",
            Publisher::XnasItchXnas => "XNAS.ITCH.XNAS",
            Publisher::XbosItchXbos => "XBOS.ITCH.XBOS",
            Publisher::XpsxItchXpsx => "XPSX.ITCH.XPSX",
            Publisher::BatsPitchBats => "BATS.PITCH.BATS",
            Publisher::BatyPitchBats => "BATY.PITCH.BATS",
            Publisher::EdgaPitchEdga => "EDGA.PITCH.EDGA",
            Publisher::EdgxPitchEdgx => "EDGX.PITCH.EDGX",
            Publisher::XnysPillarXnys => "XNYS.PILLAR.XNYS",
            Publisher::XcisPillarXcis => "XCIS.PILLAR.XCIS",
            Publisher::XasePillarXase => "XASE.PILLAR.XASE",
            Publisher::XchiPillarXchi => "XCHI.PILLAR.XCHI",
            Publisher::XcisBboXcis => "XCIS.BBO.XCIS",
            Publisher::XcisTradesXcis => "XCIS.TRADES.XCIS",
            Publisher::MemxMemoirMemx => "MEMX.MEMOIR.MEMX",
            Publisher::EprlDomEprl => "EPRL.DOM.EPRL",
            Publisher::FinnNlsFinn => "FINN.NLS.FINN",
            Publisher::FinnNlsFinc => "FINN.NLS.FINC",
            Publisher::FinyTradesFiny => "FINY.TRADES.FINY",
            Publisher::OpraPillarAmxo => "OPRA.PILLAR.AMXO",
            Publisher::OpraPillarXbox => "OPRA.PILLAR.XBOX",
            Publisher::OpraPillarXcbo => "OPRA.PILLAR.XCBO",
            Publisher::OpraPillarEmld => "OPRA.PILLAR.EMLD",
            Publisher::OpraPillarEdgo => "OPRA.PILLAR.EDGO",
            Publisher::OpraPillarGmni => "OPRA.PILLAR.GMNI",
            Publisher::OpraPillarXisx => "OPRA.PILLAR.XISX",
            Publisher::OpraPillarMcry => "OPRA.PILLAR.MCRY",
            Publisher::OpraPillarXmio => "OPRA.PILLAR.XMIO",
            Publisher::OpraPillarArco => "OPRA.PILLAR.ARCO",
            Publisher::OpraPillarOpra => "OPRA.PILLAR.OPRA",
            Publisher::OpraPillarMprl => "OPRA.PILLAR.MPRL",
            Publisher::OpraPillarXndq => "OPRA.PILLAR.XNDQ",
            Publisher::OpraPillarXbxo => "OPRA.PILLAR.XBXO",
            Publisher::OpraPillarC2Ox => "OPRA.PILLAR.C2OX",
            Publisher::OpraPillarXphl => "OPRA.PILLAR.XPHL",
            Publisher::OpraPillarBato => "OPRA.PILLAR.BATO",
        }
    }
}

impl AsRef<str> for Publisher {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::str::FromStr for Publisher {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self, Self::Err> {
        match s {
            "GLBX.MDP3.GLBX" => Ok(Publisher::GlbxMdp3Glbx),
            "XNAS.ITCH.XNAS" => Ok(Publisher::XnasItchXnas),
            "XBOS.ITCH.XBOS" => Ok(Publisher::XbosItchXbos),
            "XPSX.ITCH.XPSX" => Ok(Publisher::XpsxItchXpsx),
            "BATS.PITCH.BATS" => Ok(Publisher::BatsPitchBats),
            "BATY.PITCH.BATS" => Ok(Publisher::BatyPitchBats),
            "EDGA.PITCH.EDGA" => Ok(Publisher::EdgaPitchEdga),
            "EDGX.PITCH.EDGX" => Ok(Publisher::EdgxPitchEdgx),
            "XNYS.PILLAR.XNYS" => Ok(Publisher::XnysPillarXnys),
            "XCIS.PILLAR.XCIS" => Ok(Publisher::XcisPillarXcis),
            "XASE.PILLAR.XASE" => Ok(Publisher::XasePillarXase),
            "XCHI.PILLAR.XCHI" => Ok(Publisher::XchiPillarXchi),
            "XCIS.BBO.XCIS" => Ok(Publisher::XcisBboXcis),
            "XCIS.TRADES.XCIS" => Ok(Publisher::XcisTradesXcis),
            "MEMX.MEMOIR.MEMX" => Ok(Publisher::MemxMemoirMemx),
            "EPRL.DOM.EPRL" => Ok(Publisher::EprlDomEprl),
            "FINN.NLS.FINN" => Ok(Publisher::FinnNlsFinn),
            "FINN.NLS.FINC" => Ok(Publisher::FinnNlsFinc),
            "FINY.TRADES.FINY" => Ok(Publisher::FinyTradesFiny),
            "OPRA.PILLAR.AMXO" => Ok(Publisher::OpraPillarAmxo),
            "OPRA.PILLAR.XBOX" => Ok(Publisher::OpraPillarXbox),
            "OPRA.PILLAR.XCBO" => Ok(Publisher::OpraPillarXcbo),
            "OPRA.PILLAR.EMLD" => Ok(Publisher::OpraPillarEmld),
            "OPRA.PILLAR.EDGO" => Ok(Publisher::OpraPillarEdgo),
            "OPRA.PILLAR.GMNI" => Ok(Publisher::OpraPillarGmni),
            "OPRA.PILLAR.XISX" => Ok(Publisher::OpraPillarXisx),
            "OPRA.PILLAR.MCRY" => Ok(Publisher::OpraPillarMcry),
            "OPRA.PILLAR.XMIO" => Ok(Publisher::OpraPillarXmio),
            "OPRA.PILLAR.ARCO" => Ok(Publisher::OpraPillarArco),
            "OPRA.PILLAR.OPRA" => Ok(Publisher::OpraPillarOpra),
            "OPRA.PILLAR.MPRL" => Ok(Publisher::OpraPillarMprl),
            "OPRA.PILLAR.XNDQ" => Ok(Publisher::OpraPillarXndq),
            "OPRA.PILLAR.XBXO" => Ok(Publisher::OpraPillarXbxo),
            "OPRA.PILLAR.C2OX" => Ok(Publisher::OpraPillarC2Ox),
            "OPRA.PILLAR.XPHL" => Ok(Publisher::OpraPillarXphl),
            "OPRA.PILLAR.BATO" => Ok(Publisher::OpraPillarBato),
            _ => Err(anyhow::format_err!(
                "String doesn't match any valid Publisher"
            )),
        }
    }
}
impl Publisher {
    /// Get a Publisher's Venue.
    pub const fn venue(&self) -> Venue {
        match self {
            Publisher::GlbxMdp3Glbx => Venue::Glbx,
            Publisher::XnasItchXnas => Venue::Xnas,
            Publisher::XbosItchXbos => Venue::Xbos,
            Publisher::XpsxItchXpsx => Venue::Xpsx,
            Publisher::BatsPitchBats => Venue::Bats,
            Publisher::BatyPitchBats => Venue::Baty,
            Publisher::EdgaPitchEdga => Venue::Edga,
            Publisher::EdgxPitchEdgx => Venue::Edgx,
            Publisher::XnysPillarXnys => Venue::Xnys,
            Publisher::XcisPillarXcis => Venue::Xcis,
            Publisher::XasePillarXase => Venue::Xase,
            Publisher::XchiPillarXchi => Venue::Xchi,
            Publisher::XcisBboXcis => Venue::Xcis,
            Publisher::XcisTradesXcis => Venue::Xcis,
            Publisher::MemxMemoirMemx => Venue::Memx,
            Publisher::EprlDomEprl => Venue::Eprl,
            Publisher::FinnNlsFinn => Venue::Finn,
            Publisher::FinnNlsFinc => Venue::Finn,
            Publisher::FinyTradesFiny => Venue::Finy,
            Publisher::OpraPillarAmxo => Venue::Opra,
            Publisher::OpraPillarXbox => Venue::Opra,
            Publisher::OpraPillarXcbo => Venue::Opra,
            Publisher::OpraPillarEmld => Venue::Opra,
            Publisher::OpraPillarEdgo => Venue::Opra,
            Publisher::OpraPillarGmni => Venue::Opra,
            Publisher::OpraPillarXisx => Venue::Opra,
            Publisher::OpraPillarMcry => Venue::Opra,
            Publisher::OpraPillarXmio => Venue::Opra,
            Publisher::OpraPillarArco => Venue::Opra,
            Publisher::OpraPillarOpra => Venue::Opra,
            Publisher::OpraPillarMprl => Venue::Opra,
            Publisher::OpraPillarXndq => Venue::Opra,
            Publisher::OpraPillarXbxo => Venue::Opra,
            Publisher::OpraPillarC2Ox => Venue::Opra,
            Publisher::OpraPillarXphl => Venue::Opra,
            Publisher::OpraPillarBato => Venue::Opra,
        }
    }

    /// Get a Publisher's Dataset.
    pub const fn dataset(&self) -> Dataset {
        match self {
            Publisher::GlbxMdp3Glbx => Dataset::GlbxMdp3,
            Publisher::XnasItchXnas => Dataset::XnasItch,
            Publisher::XbosItchXbos => Dataset::XbosItch,
            Publisher::XpsxItchXpsx => Dataset::XpsxItch,
            Publisher::BatsPitchBats => Dataset::BatsPitch,
            Publisher::BatyPitchBats => Dataset::BatyPitch,
            Publisher::EdgaPitchEdga => Dataset::EdgaPitch,
            Publisher::EdgxPitchEdgx => Dataset::EdgxPitch,
            Publisher::XnysPillarXnys => Dataset::XnysPillar,
            Publisher::XcisPillarXcis => Dataset::XcisPillar,
            Publisher::XasePillarXase => Dataset::XasePillar,
            Publisher::XchiPillarXchi => Dataset::XchiPillar,
            Publisher::XcisBboXcis => Dataset::XcisBbo,
            Publisher::XcisTradesXcis => Dataset::XcisTrades,
            Publisher::MemxMemoirMemx => Dataset::MemxMemoir,
            Publisher::EprlDomEprl => Dataset::EprlDom,
            Publisher::FinnNlsFinn => Dataset::FinnNls,
            Publisher::FinnNlsFinc => Dataset::FinnNls,
            Publisher::FinyTradesFiny => Dataset::FinyTrades,
            Publisher::OpraPillarAmxo => Dataset::OpraPillar,
            Publisher::OpraPillarXbox => Dataset::OpraPillar,
            Publisher::OpraPillarXcbo => Dataset::OpraPillar,
            Publisher::OpraPillarEmld => Dataset::OpraPillar,
            Publisher::OpraPillarEdgo => Dataset::OpraPillar,
            Publisher::OpraPillarGmni => Dataset::OpraPillar,
            Publisher::OpraPillarXisx => Dataset::OpraPillar,
            Publisher::OpraPillarMcry => Dataset::OpraPillar,
            Publisher::OpraPillarXmio => Dataset::OpraPillar,
            Publisher::OpraPillarArco => Dataset::OpraPillar,
            Publisher::OpraPillarOpra => Dataset::OpraPillar,
            Publisher::OpraPillarMprl => Dataset::OpraPillar,
            Publisher::OpraPillarXndq => Dataset::OpraPillar,
            Publisher::OpraPillarXbxo => Dataset::OpraPillar,
            Publisher::OpraPillarC2Ox => Dataset::OpraPillar,
            Publisher::OpraPillarXphl => Dataset::OpraPillar,
            Publisher::OpraPillarBato => Dataset::OpraPillar,
        }
    }
}
