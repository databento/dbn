use std::fmt::{self, Display, Formatter};

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{Error, Result};

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
    /// MEMX Options Exchange
    Mxop = 37,
}

/// The number of Venue variants.
pub const VENUE_COUNT: usize = 37;

impl Venue {
    /// Convert a Venue to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Glbx => "GLBX",
            Self::Xnas => "XNAS",
            Self::Xbos => "XBOS",
            Self::Xpsx => "XPSX",
            Self::Bats => "BATS",
            Self::Baty => "BATY",
            Self::Edga => "EDGA",
            Self::Edgx => "EDGX",
            Self::Xnys => "XNYS",
            Self::Xcis => "XCIS",
            Self::Xase => "XASE",
            Self::Arcx => "ARCX",
            Self::Xchi => "XCHI",
            Self::Iexg => "IEXG",
            Self::Finn => "FINN",
            Self::Finc => "FINC",
            Self::Finy => "FINY",
            Self::Memx => "MEMX",
            Self::Eprl => "EPRL",
            Self::Amxo => "AMXO",
            Self::Xbox => "XBOX",
            Self::Xcbo => "XCBO",
            Self::Emld => "EMLD",
            Self::Edgo => "EDGO",
            Self::Gmni => "GMNI",
            Self::Xisx => "XISX",
            Self::Mcry => "MCRY",
            Self::Xmio => "XMIO",
            Self::Arco => "ARCO",
            Self::Opra => "OPRA",
            Self::Mprl => "MPRL",
            Self::Xndq => "XNDQ",
            Self::Xbxo => "XBXO",
            Self::C2Ox => "C2OX",
            Self::Xphl => "XPHL",
            Self::Bato => "BATO",
            Self::Mxop => "MXOP",
        }
    }
}

impl AsRef<str> for Venue {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for Venue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Venue {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "GLBX" => Ok(Self::Glbx),
            "XNAS" => Ok(Self::Xnas),
            "XBOS" => Ok(Self::Xbos),
            "XPSX" => Ok(Self::Xpsx),
            "BATS" => Ok(Self::Bats),
            "BATY" => Ok(Self::Baty),
            "EDGA" => Ok(Self::Edga),
            "EDGX" => Ok(Self::Edgx),
            "XNYS" => Ok(Self::Xnys),
            "XCIS" => Ok(Self::Xcis),
            "XASE" => Ok(Self::Xase),
            "ARCX" => Ok(Self::Arcx),
            "XCHI" => Ok(Self::Xchi),
            "IEXG" => Ok(Self::Iexg),
            "FINN" => Ok(Self::Finn),
            "FINC" => Ok(Self::Finc),
            "FINY" => Ok(Self::Finy),
            "MEMX" => Ok(Self::Memx),
            "EPRL" => Ok(Self::Eprl),
            "AMXO" => Ok(Self::Amxo),
            "XBOX" => Ok(Self::Xbox),
            "XCBO" => Ok(Self::Xcbo),
            "EMLD" => Ok(Self::Emld),
            "EDGO" => Ok(Self::Edgo),
            "GMNI" => Ok(Self::Gmni),
            "XISX" => Ok(Self::Xisx),
            "MCRY" => Ok(Self::Mcry),
            "XMIO" => Ok(Self::Xmio),
            "ARCO" => Ok(Self::Arco),
            "OPRA" => Ok(Self::Opra),
            "MPRL" => Ok(Self::Mprl),
            "XNDQ" => Ok(Self::Xndq),
            "XBXO" => Ok(Self::Xbxo),
            "C2OX" => Ok(Self::C2Ox),
            "XPHL" => Ok(Self::Xphl),
            "BATO" => Ok(Self::Bato),
            "MXOP" => Ok(Self::Mxop),
            _ => Err(Error::conversion::<Self>(s)),
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
    /// NYSE Arca
    ArcxPillar = 21,
    /// Investors Exchange TOPS
    IexgTops = 22,
}

/// The number of Dataset variants.
pub const DATASET_COUNT: usize = 22;

impl Dataset {
    /// Convert a Dataset to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::GlbxMdp3 => "GLBX.MDP3",
            Self::XnasItch => "XNAS.ITCH",
            Self::XbosItch => "XBOS.ITCH",
            Self::XpsxItch => "XPSX.ITCH",
            Self::BatsPitch => "BATS.PITCH",
            Self::BatyPitch => "BATY.PITCH",
            Self::EdgaPitch => "EDGA.PITCH",
            Self::EdgxPitch => "EDGX.PITCH",
            Self::XnysPillar => "XNYS.PILLAR",
            Self::XcisPillar => "XCIS.PILLAR",
            Self::XasePillar => "XASE.PILLAR",
            Self::XchiPillar => "XCHI.PILLAR",
            Self::XcisBbo => "XCIS.BBO",
            Self::XcisTrades => "XCIS.TRADES",
            Self::MemxMemoir => "MEMX.MEMOIR",
            Self::EprlDom => "EPRL.DOM",
            Self::FinnNls => "FINN.NLS",
            Self::FinyTrades => "FINY.TRADES",
            Self::OpraPillar => "OPRA.PILLAR",
            Self::DbeqBasic => "DBEQ.BASIC",
            Self::ArcxPillar => "ARCX.PILLAR",
            Self::IexgTops => "IEXG.TOPS",
        }
    }
}

impl AsRef<str> for Dataset {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for Dataset {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Dataset {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "GLBX.MDP3" => Ok(Self::GlbxMdp3),
            "XNAS.ITCH" => Ok(Self::XnasItch),
            "XBOS.ITCH" => Ok(Self::XbosItch),
            "XPSX.ITCH" => Ok(Self::XpsxItch),
            "BATS.PITCH" => Ok(Self::BatsPitch),
            "BATY.PITCH" => Ok(Self::BatyPitch),
            "EDGA.PITCH" => Ok(Self::EdgaPitch),
            "EDGX.PITCH" => Ok(Self::EdgxPitch),
            "XNYS.PILLAR" => Ok(Self::XnysPillar),
            "XCIS.PILLAR" => Ok(Self::XcisPillar),
            "XASE.PILLAR" => Ok(Self::XasePillar),
            "XCHI.PILLAR" => Ok(Self::XchiPillar),
            "XCIS.BBO" => Ok(Self::XcisBbo),
            "XCIS.TRADES" => Ok(Self::XcisTrades),
            "MEMX.MEMOIR" => Ok(Self::MemxMemoir),
            "EPRL.DOM" => Ok(Self::EprlDom),
            "FINN.NLS" => Ok(Self::FinnNls),
            "FINY.TRADES" => Ok(Self::FinyTrades),
            "OPRA.PILLAR" => Ok(Self::OpraPillar),
            "DBEQ.BASIC" => Ok(Self::DbeqBasic),
            "ARCX.PILLAR" => Ok(Self::ArcxPillar),
            "IEXG.TOPS" => Ok(Self::IexgTops),
            _ => Err(Error::conversion::<Self>(s)),
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
    BatyPitchBaty = 6,
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
    /// OPRA - MEMX OPTIONS EXCHANGE
    OpraPillarMxop = 37,
    /// Investors Exchange TOPS
    IexgTopsIexg = 38,
    /// DBEQ Basic - Nyse Chicago
    DbeqBasicXchi = 39,
    /// DBEQ Basic - Nyse National
    DbeqBasicXcis = 40,
    /// DBEQ Basic - Investors Exchange
    DbeqBasicIexg = 41,
    /// DBEQ Basic - MIAX Pearl
    DbeqBasicEprl = 42,
}

/// The number of Publisher variants.
pub const PUBLISHER_COUNT: usize = 42;

impl Publisher {
    /// Convert a Publisher to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::GlbxMdp3Glbx => "GLBX.MDP3.GLBX",
            Self::XnasItchXnas => "XNAS.ITCH.XNAS",
            Self::XbosItchXbos => "XBOS.ITCH.XBOS",
            Self::XpsxItchXpsx => "XPSX.ITCH.XPSX",
            Self::BatsPitchBats => "BATS.PITCH.BATS",
            Self::BatyPitchBaty => "BATY.PITCH.BATY",
            Self::EdgaPitchEdga => "EDGA.PITCH.EDGA",
            Self::EdgxPitchEdgx => "EDGX.PITCH.EDGX",
            Self::XnysPillarXnys => "XNYS.PILLAR.XNYS",
            Self::XcisPillarXcis => "XCIS.PILLAR.XCIS",
            Self::XasePillarXase => "XASE.PILLAR.XASE",
            Self::XchiPillarXchi => "XCHI.PILLAR.XCHI",
            Self::XcisBboXcis => "XCIS.BBO.XCIS",
            Self::XcisTradesXcis => "XCIS.TRADES.XCIS",
            Self::MemxMemoirMemx => "MEMX.MEMOIR.MEMX",
            Self::EprlDomEprl => "EPRL.DOM.EPRL",
            Self::FinnNlsFinn => "FINN.NLS.FINN",
            Self::FinnNlsFinc => "FINN.NLS.FINC",
            Self::FinyTradesFiny => "FINY.TRADES.FINY",
            Self::OpraPillarAmxo => "OPRA.PILLAR.AMXO",
            Self::OpraPillarXbox => "OPRA.PILLAR.XBOX",
            Self::OpraPillarXcbo => "OPRA.PILLAR.XCBO",
            Self::OpraPillarEmld => "OPRA.PILLAR.EMLD",
            Self::OpraPillarEdgo => "OPRA.PILLAR.EDGO",
            Self::OpraPillarGmni => "OPRA.PILLAR.GMNI",
            Self::OpraPillarXisx => "OPRA.PILLAR.XISX",
            Self::OpraPillarMcry => "OPRA.PILLAR.MCRY",
            Self::OpraPillarXmio => "OPRA.PILLAR.XMIO",
            Self::OpraPillarArco => "OPRA.PILLAR.ARCO",
            Self::OpraPillarOpra => "OPRA.PILLAR.OPRA",
            Self::OpraPillarMprl => "OPRA.PILLAR.MPRL",
            Self::OpraPillarXndq => "OPRA.PILLAR.XNDQ",
            Self::OpraPillarXbxo => "OPRA.PILLAR.XBXO",
            Self::OpraPillarC2Ox => "OPRA.PILLAR.C2OX",
            Self::OpraPillarXphl => "OPRA.PILLAR.XPHL",
            Self::OpraPillarBato => "OPRA.PILLAR.BATO",
            Self::OpraPillarMxop => "OPRA.PILLAR.MXOP",
            Self::IexgTopsIexg => "IEXG.TOPS.IEXG",
            Self::DbeqBasicXchi => "DBEQ.BASIC.XCHI",
            Self::DbeqBasicXcis => "DBEQ.BASIC.XCIS",
            Self::DbeqBasicIexg => "DBEQ.BASIC.IEXG",
            Self::DbeqBasicEprl => "DBEQ.BASIC.EPRL",
        }
    }

    /// Get a Publisher's Venue.
    pub const fn venue(&self) -> Venue {
        match self {
            Self::GlbxMdp3Glbx => Venue::Glbx,
            Self::XnasItchXnas => Venue::Xnas,
            Self::XbosItchXbos => Venue::Xbos,
            Self::XpsxItchXpsx => Venue::Xpsx,
            Self::BatsPitchBats => Venue::Bats,
            Self::BatyPitchBaty => Venue::Baty,
            Self::EdgaPitchEdga => Venue::Edga,
            Self::EdgxPitchEdgx => Venue::Edgx,
            Self::XnysPillarXnys => Venue::Xnys,
            Self::XcisPillarXcis => Venue::Xcis,
            Self::XasePillarXase => Venue::Xase,
            Self::XchiPillarXchi => Venue::Xchi,
            Self::XcisBboXcis => Venue::Xcis,
            Self::XcisTradesXcis => Venue::Xcis,
            Self::MemxMemoirMemx => Venue::Memx,
            Self::EprlDomEprl => Venue::Eprl,
            Self::FinnNlsFinn => Venue::Finn,
            Self::FinnNlsFinc => Venue::Finc,
            Self::FinyTradesFiny => Venue::Finy,
            Self::OpraPillarAmxo => Venue::Amxo,
            Self::OpraPillarXbox => Venue::Xbox,
            Self::OpraPillarXcbo => Venue::Xcbo,
            Self::OpraPillarEmld => Venue::Emld,
            Self::OpraPillarEdgo => Venue::Edgo,
            Self::OpraPillarGmni => Venue::Gmni,
            Self::OpraPillarXisx => Venue::Xisx,
            Self::OpraPillarMcry => Venue::Mcry,
            Self::OpraPillarXmio => Venue::Xmio,
            Self::OpraPillarArco => Venue::Arco,
            Self::OpraPillarOpra => Venue::Opra,
            Self::OpraPillarMprl => Venue::Mprl,
            Self::OpraPillarXndq => Venue::Xndq,
            Self::OpraPillarXbxo => Venue::Xbxo,
            Self::OpraPillarC2Ox => Venue::C2Ox,
            Self::OpraPillarXphl => Venue::Xphl,
            Self::OpraPillarBato => Venue::Bato,
            Self::OpraPillarMxop => Venue::Mxop,
            Self::IexgTopsIexg => Venue::Iexg,
            Self::DbeqBasicXchi => Venue::Xchi,
            Self::DbeqBasicXcis => Venue::Xcis,
            Self::DbeqBasicIexg => Venue::Iexg,
            Self::DbeqBasicEprl => Venue::Eprl,
        }
    }

    /// Get a Publisher's Dataset.
    pub const fn dataset(&self) -> Dataset {
        match self {
            Self::GlbxMdp3Glbx => Dataset::GlbxMdp3,
            Self::XnasItchXnas => Dataset::XnasItch,
            Self::XbosItchXbos => Dataset::XbosItch,
            Self::XpsxItchXpsx => Dataset::XpsxItch,
            Self::BatsPitchBats => Dataset::BatsPitch,
            Self::BatyPitchBaty => Dataset::BatyPitch,
            Self::EdgaPitchEdga => Dataset::EdgaPitch,
            Self::EdgxPitchEdgx => Dataset::EdgxPitch,
            Self::XnysPillarXnys => Dataset::XnysPillar,
            Self::XcisPillarXcis => Dataset::XcisPillar,
            Self::XasePillarXase => Dataset::XasePillar,
            Self::XchiPillarXchi => Dataset::XchiPillar,
            Self::XcisBboXcis => Dataset::XcisBbo,
            Self::XcisTradesXcis => Dataset::XcisTrades,
            Self::MemxMemoirMemx => Dataset::MemxMemoir,
            Self::EprlDomEprl => Dataset::EprlDom,
            Self::FinnNlsFinn => Dataset::FinnNls,
            Self::FinnNlsFinc => Dataset::FinnNls,
            Self::FinyTradesFiny => Dataset::FinyTrades,
            Self::OpraPillarAmxo => Dataset::OpraPillar,
            Self::OpraPillarXbox => Dataset::OpraPillar,
            Self::OpraPillarXcbo => Dataset::OpraPillar,
            Self::OpraPillarEmld => Dataset::OpraPillar,
            Self::OpraPillarEdgo => Dataset::OpraPillar,
            Self::OpraPillarGmni => Dataset::OpraPillar,
            Self::OpraPillarXisx => Dataset::OpraPillar,
            Self::OpraPillarMcry => Dataset::OpraPillar,
            Self::OpraPillarXmio => Dataset::OpraPillar,
            Self::OpraPillarArco => Dataset::OpraPillar,
            Self::OpraPillarOpra => Dataset::OpraPillar,
            Self::OpraPillarMprl => Dataset::OpraPillar,
            Self::OpraPillarXndq => Dataset::OpraPillar,
            Self::OpraPillarXbxo => Dataset::OpraPillar,
            Self::OpraPillarC2Ox => Dataset::OpraPillar,
            Self::OpraPillarXphl => Dataset::OpraPillar,
            Self::OpraPillarBato => Dataset::OpraPillar,
            Self::OpraPillarMxop => Dataset::OpraPillar,
            Self::IexgTopsIexg => Dataset::IexgTops,
            Self::DbeqBasicXchi => Dataset::DbeqBasic,
            Self::DbeqBasicXcis => Dataset::DbeqBasic,
            Self::DbeqBasicIexg => Dataset::DbeqBasic,
            Self::DbeqBasicEprl => Dataset::DbeqBasic,
        }
    }
}

impl AsRef<str> for Publisher {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for Publisher {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Publisher {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "GLBX.MDP3.GLBX" => Ok(Self::GlbxMdp3Glbx),
            "XNAS.ITCH.XNAS" => Ok(Self::XnasItchXnas),
            "XBOS.ITCH.XBOS" => Ok(Self::XbosItchXbos),
            "XPSX.ITCH.XPSX" => Ok(Self::XpsxItchXpsx),
            "BATS.PITCH.BATS" => Ok(Self::BatsPitchBats),
            "BATY.PITCH.BATY" => Ok(Self::BatyPitchBaty),
            "EDGA.PITCH.EDGA" => Ok(Self::EdgaPitchEdga),
            "EDGX.PITCH.EDGX" => Ok(Self::EdgxPitchEdgx),
            "XNYS.PILLAR.XNYS" => Ok(Self::XnysPillarXnys),
            "XCIS.PILLAR.XCIS" => Ok(Self::XcisPillarXcis),
            "XASE.PILLAR.XASE" => Ok(Self::XasePillarXase),
            "XCHI.PILLAR.XCHI" => Ok(Self::XchiPillarXchi),
            "XCIS.BBO.XCIS" => Ok(Self::XcisBboXcis),
            "XCIS.TRADES.XCIS" => Ok(Self::XcisTradesXcis),
            "MEMX.MEMOIR.MEMX" => Ok(Self::MemxMemoirMemx),
            "EPRL.DOM.EPRL" => Ok(Self::EprlDomEprl),
            "FINN.NLS.FINN" => Ok(Self::FinnNlsFinn),
            "FINN.NLS.FINC" => Ok(Self::FinnNlsFinc),
            "FINY.TRADES.FINY" => Ok(Self::FinyTradesFiny),
            "OPRA.PILLAR.AMXO" => Ok(Self::OpraPillarAmxo),
            "OPRA.PILLAR.XBOX" => Ok(Self::OpraPillarXbox),
            "OPRA.PILLAR.XCBO" => Ok(Self::OpraPillarXcbo),
            "OPRA.PILLAR.EMLD" => Ok(Self::OpraPillarEmld),
            "OPRA.PILLAR.EDGO" => Ok(Self::OpraPillarEdgo),
            "OPRA.PILLAR.GMNI" => Ok(Self::OpraPillarGmni),
            "OPRA.PILLAR.XISX" => Ok(Self::OpraPillarXisx),
            "OPRA.PILLAR.MCRY" => Ok(Self::OpraPillarMcry),
            "OPRA.PILLAR.XMIO" => Ok(Self::OpraPillarXmio),
            "OPRA.PILLAR.ARCO" => Ok(Self::OpraPillarArco),
            "OPRA.PILLAR.OPRA" => Ok(Self::OpraPillarOpra),
            "OPRA.PILLAR.MPRL" => Ok(Self::OpraPillarMprl),
            "OPRA.PILLAR.XNDQ" => Ok(Self::OpraPillarXndq),
            "OPRA.PILLAR.XBXO" => Ok(Self::OpraPillarXbxo),
            "OPRA.PILLAR.C2OX" => Ok(Self::OpraPillarC2Ox),
            "OPRA.PILLAR.XPHL" => Ok(Self::OpraPillarXphl),
            "OPRA.PILLAR.BATO" => Ok(Self::OpraPillarBato),
            "OPRA.PILLAR.MXOP" => Ok(Self::OpraPillarMxop),
            "IEXG.TOPS.IEXG" => Ok(Self::IexgTopsIexg),
            "DBEQ.BASIC.XCHI" => Ok(Self::DbeqBasicXchi),
            "DBEQ.BASIC.XCIS" => Ok(Self::DbeqBasicXcis),
            "DBEQ.BASIC.IEXG" => Ok(Self::DbeqBasicIexg),
            "DBEQ.BASIC.EPRL" => Ok(Self::DbeqBasicEprl),
            _ => Err(Error::conversion::<Self>(s)),
        }
    }
}
