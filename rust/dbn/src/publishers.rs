//! Enumerations for different data sources, venues, and publishers.

use std::fmt::{self, Display, Formatter};

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{Error, Result};

/// A trading execution venue.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u16)]
pub enum Venue {
    /// CME Globex
    Glbx = 1,
    /// Nasdaq - All Markets
    Xnas = 2,
    /// Nasdaq OMX BX
    Xbos = 3,
    /// Nasdaq OMX PSX
    Xpsx = 4,
    /// Cboe BZX U.S. Equities Exchange
    Bats = 5,
    /// Cboe BYX U.S. Equities Exchange
    Baty = 6,
    /// Cboe EDGA U.S. Equities Exchange
    Edga = 7,
    /// Cboe EDGX U.S. Equities Exchange
    Edgx = 8,
    /// New York Stock Exchange, Inc.
    Xnys = 9,
    /// NYSE National, Inc.
    Xcis = 10,
    /// NYSE MKT LLC
    Xase = 11,
    /// NYSE Arca
    Arcx = 12,
    /// NYSE Chicago, Inc.
    Xchi = 13,
    /// Investors Exchange
    Iexg = 14,
    /// FINRA/Nasdaq TRF Carteret
    Finn = 15,
    /// FINRA/Nasdaq TRF Chicago
    Finc = 16,
    /// FINRA/NYSE TRF
    Finy = 17,
    /// MEMX LLC Equities
    Memx = 18,
    /// MIAX Pearl Equities
    Eprl = 19,
    /// NYSE American Options
    Amxo = 20,
    /// BOX Options Exchange
    Xbox = 21,
    /// Cboe Options Exchange
    Xcbo = 22,
    /// MIAX Emerald
    Emld = 23,
    /// Cboe EDGX Options Exchange
    Edgo = 24,
    /// ISE Gemini Exchange
    Gmni = 25,
    /// International Securities Exchange, LLC
    Xisx = 26,
    /// ISE Mercury, LLC
    Mcry = 27,
    /// Miami International Securities Exchange
    Xmio = 28,
    /// NYSE Arca Options
    Arco = 29,
    /// Options Price Reporting Authority
    Opra = 30,
    /// MIAX Pearl
    Mprl = 31,
    /// Nasdaq Options Market
    Xndq = 32,
    /// Nasdaq OMX BX Options
    Xbxo = 33,
    /// Cboe C2 Options Exchange
    C2Ox = 34,
    /// Nasdaq OMX PHLX
    Xphl = 35,
    /// Cboe BZX Options Exchange
    Bato = 36,
    /// MEMX LLC Options
    Mxop = 37,
    /// ICE Futures Europe (Commodities)
    Ifeu = 38,
    /// ICE Endex
    Ndex = 39,
    /// Databento Equities - Consolidated
    Dbeq = 40,
}

/// The number of Venue variants.
pub const VENUE_COUNT: usize = 40;

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
            Self::Ifeu => "IFEU",
            Self::Ndex => "NDEX",
            Self::Dbeq => "DBEQ",
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
            "IFEU" => Ok(Self::Ifeu),
            "NDEX" => Ok(Self::Ndex),
            "DBEQ" => Ok(Self::Dbeq),
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
    /// Nasdaq TotalView-ITCH
    XnasItch = 2,
    /// Nasdaq BX TotalView-ITCH
    XbosItch = 3,
    /// Nasdaq PSX TotalView-ITCH
    XpsxItch = 4,
    /// Cboe BZX Depth Pitch
    BatsPitch = 5,
    /// Cboe BYX Depth Pitch
    BatyPitch = 6,
    /// Cboe EDGA Depth Pitch
    EdgaPitch = 7,
    /// Cboe EDGX Depth Pitch
    EdgxPitch = 8,
    /// NYSE Integrated
    XnysPillar = 9,
    /// NYSE National Integrated
    XcisPillar = 10,
    /// NYSE American Integrated
    XasePillar = 11,
    /// NYSE Chicago Integrated
    XchiPillar = 12,
    /// NYSE National BBO
    XcisBbo = 13,
    /// NYSE National Trades
    XcisTrades = 14,
    /// MEMX Memoir Depth
    MemxMemoir = 15,
    /// MIAX Pearl Depth
    EprlDom = 16,
    /// FINRA/Nasdaq TRF
    FinnNls = 17,
    /// FINRA/NYSE TRF
    FinyTrades = 18,
    /// OPRA Binary
    OpraPillar = 19,
    /// Databento Equities Basic
    DbeqBasic = 20,
    /// NYSE Arca Integrated
    ArcxPillar = 21,
    /// IEX TOPS
    IexgTops = 22,
    /// Databento Equities Plus
    DbeqPlus = 23,
    /// NYSE BBO
    XnysBbo = 24,
    /// NYSE Trades
    XnysTrades = 25,
    /// Nasdaq QBBO
    XnasQbbo = 26,
    /// Nasdaq NLS
    XnasNls = 27,
    /// ICE Futures Europe (Commodities) iMpact
    IfeuImpact = 28,
    /// ICE Endex iMpact
    NdexImpact = 29,
}

/// The number of Dataset variants.
pub const DATASET_COUNT: usize = 29;

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
            Self::DbeqPlus => "DBEQ.PLUS",
            Self::XnysBbo => "XNYS.BBO",
            Self::XnysTrades => "XNYS.TRADES",
            Self::XnasQbbo => "XNAS.QBBO",
            Self::XnasNls => "XNAS.NLS",
            Self::IfeuImpact => "IFEU.IMPACT",
            Self::NdexImpact => "NDEX.IMPACT",
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
            "DBEQ.PLUS" => Ok(Self::DbeqPlus),
            "XNYS.BBO" => Ok(Self::XnysBbo),
            "XNYS.TRADES" => Ok(Self::XnysTrades),
            "XNAS.QBBO" => Ok(Self::XnasQbbo),
            "XNAS.NLS" => Ok(Self::XnasNls),
            "IFEU.IMPACT" => Ok(Self::IfeuImpact),
            "NDEX.IMPACT" => Ok(Self::NdexImpact),
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
    /// Nasdaq TotalView-ITCH
    XnasItchXnas = 2,
    /// Nasdaq BX TotalView-ITCH
    XbosItchXbos = 3,
    /// Nasdaq PSX TotalView-ITCH
    XpsxItchXpsx = 4,
    /// Cboe BZX Depth Pitch
    BatsPitchBats = 5,
    /// Cboe BYX Depth Pitch
    BatyPitchBaty = 6,
    /// Cboe EDGA Depth Pitch
    EdgaPitchEdga = 7,
    /// Cboe EDGX Depth Pitch
    EdgxPitchEdgx = 8,
    /// NYSE Integrated
    XnysPillarXnys = 9,
    /// NYSE National Integrated
    XcisPillarXcis = 10,
    /// NYSE American Integrated
    XasePillarXase = 11,
    /// NYSE Chicago Integrated
    XchiPillarXchi = 12,
    /// NYSE National BBO
    XcisBboXcis = 13,
    /// NYSE National Trades
    XcisTradesXcis = 14,
    /// MEMX Memoir Depth
    MemxMemoirMemx = 15,
    /// MIAX Pearl Depth
    EprlDomEprl = 16,
    /// FINRA/Nasdaq TRF Carteret
    FinnNlsFinn = 17,
    /// FINRA/Nasdaq TRF Chicago
    FinnNlsFinc = 18,
    /// FINRA/NYSE TRF
    FinyTradesFiny = 19,
    /// OPRA - NYSE American
    OpraPillarAmxo = 20,
    /// OPRA - Boston Options Exchange
    OpraPillarXbox = 21,
    /// OPRA - Cboe Options Exchange
    OpraPillarXcbo = 22,
    /// OPRA - MIAX Emerald
    OpraPillarEmld = 23,
    /// OPRA - Cboe EDGX Options Exchange
    OpraPillarEdgo = 24,
    /// OPRA - Nasdaq GEMX
    OpraPillarGmni = 25,
    /// OPRA - Nasdaq ISE
    OpraPillarXisx = 26,
    /// OPRA - Nasdaq MRX
    OpraPillarMcry = 27,
    /// OPRA - Miami International Securities
    OpraPillarXmio = 28,
    /// OPRA - NYSE Arca
    OpraPillarArco = 29,
    /// OPRA - Options Price Reporting Authority
    OpraPillarOpra = 30,
    /// OPRA - MIAX Pearl
    OpraPillarMprl = 31,
    /// OPRA - Nasdaq Options Market
    OpraPillarXndq = 32,
    /// OPRA - Nasdaq BX Options
    OpraPillarXbxo = 33,
    /// OPRA - Cboe C2 Options Exchange
    OpraPillarC2Ox = 34,
    /// OPRA - Nasdaq PHLX
    OpraPillarXphl = 35,
    /// OPRA - Cboe BZX Options
    OpraPillarBato = 36,
    /// OPRA - MEMX Options Exchange
    OpraPillarMxop = 37,
    /// IEX TOPS
    IexgTopsIexg = 38,
    /// DBEQ Basic - NYSE Chicago
    DbeqBasicXchi = 39,
    /// DBEQ Basic - NYSE National
    DbeqBasicXcis = 40,
    /// DBEQ Basic - IEX
    DbeqBasicIexg = 41,
    /// DBEQ Basic - MIAX Pearl
    DbeqBasicEprl = 42,
    /// NYSE Arca Integrated
    ArcxPillarArcx = 43,
    /// NYSE BBO
    XnysBboXnys = 44,
    /// NYSE Trades
    XnysTradesXnys = 45,
    /// Nasdaq QBBO
    XnasQbboXnas = 46,
    /// Nasdaq Trades
    XnasNlsXnas = 47,
    /// DBEQ Plus - NYSE Chicago
    DbeqPlusXchi = 48,
    /// DBEQ Plus - NYSE National
    DbeqPlusXcis = 49,
    /// DBEQ Plus - IEX
    DbeqPlusIexg = 50,
    /// DBEQ Plus - MIAX Pearl
    DbeqPlusEprl = 51,
    /// DBEQ Plus - Nasdaq
    DbeqPlusXnas = 52,
    /// DBEQ Plus - NYSE
    DbeqPlusXnys = 53,
    /// DBEQ Plus - FINRA/NYSE TRF
    DbeqPlusFinn = 54,
    /// DBEQ Plus - FINRA/Nasdaq TRF Carteret
    DbeqPlusFiny = 55,
    /// DBEQ Plus - FINRA/Nasdaq TRF Chicago
    DbeqPlusFinc = 56,
    /// ICE Futures Europe (Commodities)
    IfeuImpactIfeu = 57,
    /// ICE Endex
    NdexImpactNdex = 58,
    /// DBEQ Basic - Consolidated
    DbeqBasicDbeq = 59,
    /// DBEQ Plus - Consolidated
    DbeqPlusDbeq = 60,
}

/// The number of Publisher variants.
pub const PUBLISHER_COUNT: usize = 60;

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
            Self::ArcxPillarArcx => "ARCX.PILLAR.ARCX",
            Self::XnysBboXnys => "XNYS.BBO.XNYS",
            Self::XnysTradesXnys => "XNYS.TRADES.XNYS",
            Self::XnasQbboXnas => "XNAS.QBBO.XNAS",
            Self::XnasNlsXnas => "XNAS.NLS.XNAS",
            Self::DbeqPlusXchi => "DBEQ.PLUS.XCHI",
            Self::DbeqPlusXcis => "DBEQ.PLUS.XCIS",
            Self::DbeqPlusIexg => "DBEQ.PLUS.IEXG",
            Self::DbeqPlusEprl => "DBEQ.PLUS.EPRL",
            Self::DbeqPlusXnas => "DBEQ.PLUS.XNAS",
            Self::DbeqPlusXnys => "DBEQ.PLUS.XNYS",
            Self::DbeqPlusFinn => "DBEQ.PLUS.FINN",
            Self::DbeqPlusFiny => "DBEQ.PLUS.FINY",
            Self::DbeqPlusFinc => "DBEQ.PLUS.FINC",
            Self::IfeuImpactIfeu => "IFEU.IMPACT.IFEU",
            Self::NdexImpactNdex => "NDEX.IMPACT.NDEX",
            Self::DbeqBasicDbeq => "DBEQ.BASIC.DBEQ",
            Self::DbeqPlusDbeq => "DBEQ.PLUS.DBEQ",
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
            Self::ArcxPillarArcx => Venue::Arcx,
            Self::XnysBboXnys => Venue::Xnys,
            Self::XnysTradesXnys => Venue::Xnys,
            Self::XnasQbboXnas => Venue::Xnas,
            Self::XnasNlsXnas => Venue::Xnas,
            Self::DbeqPlusXchi => Venue::Xchi,
            Self::DbeqPlusXcis => Venue::Xcis,
            Self::DbeqPlusIexg => Venue::Iexg,
            Self::DbeqPlusEprl => Venue::Eprl,
            Self::DbeqPlusXnas => Venue::Xnas,
            Self::DbeqPlusXnys => Venue::Xnys,
            Self::DbeqPlusFinn => Venue::Finn,
            Self::DbeqPlusFiny => Venue::Finy,
            Self::DbeqPlusFinc => Venue::Finc,
            Self::IfeuImpactIfeu => Venue::Ifeu,
            Self::NdexImpactNdex => Venue::Ndex,
            Self::DbeqBasicDbeq => Venue::Dbeq,
            Self::DbeqPlusDbeq => Venue::Dbeq,
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
            Self::ArcxPillarArcx => Dataset::ArcxPillar,
            Self::XnysBboXnys => Dataset::XnysBbo,
            Self::XnysTradesXnys => Dataset::XnysTrades,
            Self::XnasQbboXnas => Dataset::XnasQbbo,
            Self::XnasNlsXnas => Dataset::XnasNls,
            Self::DbeqPlusXchi => Dataset::DbeqPlus,
            Self::DbeqPlusXcis => Dataset::DbeqPlus,
            Self::DbeqPlusIexg => Dataset::DbeqPlus,
            Self::DbeqPlusEprl => Dataset::DbeqPlus,
            Self::DbeqPlusXnas => Dataset::DbeqPlus,
            Self::DbeqPlusXnys => Dataset::DbeqPlus,
            Self::DbeqPlusFinn => Dataset::DbeqPlus,
            Self::DbeqPlusFiny => Dataset::DbeqPlus,
            Self::DbeqPlusFinc => Dataset::DbeqPlus,
            Self::IfeuImpactIfeu => Dataset::IfeuImpact,
            Self::NdexImpactNdex => Dataset::NdexImpact,
            Self::DbeqBasicDbeq => Dataset::DbeqBasic,
            Self::DbeqPlusDbeq => Dataset::DbeqPlus,
        }
    }

    /// Construct a Publisher from its components.
    /// # Errors
    /// Returns an error if there's no `Publisher` with the corresponding `Dataset`/`Venue` combination.
    pub fn from_dataset_venue(dataset: Dataset, venue: Venue) -> Result<Self> {
        match (dataset, venue) {
            (Dataset::GlbxMdp3, Venue::Glbx) => Ok(Self::GlbxMdp3Glbx),
            (Dataset::XnasItch, Venue::Xnas) => Ok(Self::XnasItchXnas),
            (Dataset::XbosItch, Venue::Xbos) => Ok(Self::XbosItchXbos),
            (Dataset::XpsxItch, Venue::Xpsx) => Ok(Self::XpsxItchXpsx),
            (Dataset::BatsPitch, Venue::Bats) => Ok(Self::BatsPitchBats),
            (Dataset::BatyPitch, Venue::Baty) => Ok(Self::BatyPitchBaty),
            (Dataset::EdgaPitch, Venue::Edga) => Ok(Self::EdgaPitchEdga),
            (Dataset::EdgxPitch, Venue::Edgx) => Ok(Self::EdgxPitchEdgx),
            (Dataset::XnysPillar, Venue::Xnys) => Ok(Self::XnysPillarXnys),
            (Dataset::XcisPillar, Venue::Xcis) => Ok(Self::XcisPillarXcis),
            (Dataset::XasePillar, Venue::Xase) => Ok(Self::XasePillarXase),
            (Dataset::XchiPillar, Venue::Xchi) => Ok(Self::XchiPillarXchi),
            (Dataset::XcisBbo, Venue::Xcis) => Ok(Self::XcisBboXcis),
            (Dataset::XcisTrades, Venue::Xcis) => Ok(Self::XcisTradesXcis),
            (Dataset::MemxMemoir, Venue::Memx) => Ok(Self::MemxMemoirMemx),
            (Dataset::EprlDom, Venue::Eprl) => Ok(Self::EprlDomEprl),
            (Dataset::FinnNls, Venue::Finn) => Ok(Self::FinnNlsFinn),
            (Dataset::FinnNls, Venue::Finc) => Ok(Self::FinnNlsFinc),
            (Dataset::FinyTrades, Venue::Finy) => Ok(Self::FinyTradesFiny),
            (Dataset::OpraPillar, Venue::Amxo) => Ok(Self::OpraPillarAmxo),
            (Dataset::OpraPillar, Venue::Xbox) => Ok(Self::OpraPillarXbox),
            (Dataset::OpraPillar, Venue::Xcbo) => Ok(Self::OpraPillarXcbo),
            (Dataset::OpraPillar, Venue::Emld) => Ok(Self::OpraPillarEmld),
            (Dataset::OpraPillar, Venue::Edgo) => Ok(Self::OpraPillarEdgo),
            (Dataset::OpraPillar, Venue::Gmni) => Ok(Self::OpraPillarGmni),
            (Dataset::OpraPillar, Venue::Xisx) => Ok(Self::OpraPillarXisx),
            (Dataset::OpraPillar, Venue::Mcry) => Ok(Self::OpraPillarMcry),
            (Dataset::OpraPillar, Venue::Xmio) => Ok(Self::OpraPillarXmio),
            (Dataset::OpraPillar, Venue::Arco) => Ok(Self::OpraPillarArco),
            (Dataset::OpraPillar, Venue::Opra) => Ok(Self::OpraPillarOpra),
            (Dataset::OpraPillar, Venue::Mprl) => Ok(Self::OpraPillarMprl),
            (Dataset::OpraPillar, Venue::Xndq) => Ok(Self::OpraPillarXndq),
            (Dataset::OpraPillar, Venue::Xbxo) => Ok(Self::OpraPillarXbxo),
            (Dataset::OpraPillar, Venue::C2Ox) => Ok(Self::OpraPillarC2Ox),
            (Dataset::OpraPillar, Venue::Xphl) => Ok(Self::OpraPillarXphl),
            (Dataset::OpraPillar, Venue::Bato) => Ok(Self::OpraPillarBato),
            (Dataset::OpraPillar, Venue::Mxop) => Ok(Self::OpraPillarMxop),
            (Dataset::IexgTops, Venue::Iexg) => Ok(Self::IexgTopsIexg),
            (Dataset::DbeqBasic, Venue::Xchi) => Ok(Self::DbeqBasicXchi),
            (Dataset::DbeqBasic, Venue::Xcis) => Ok(Self::DbeqBasicXcis),
            (Dataset::DbeqBasic, Venue::Iexg) => Ok(Self::DbeqBasicIexg),
            (Dataset::DbeqBasic, Venue::Eprl) => Ok(Self::DbeqBasicEprl),
            (Dataset::ArcxPillar, Venue::Arcx) => Ok(Self::ArcxPillarArcx),
            (Dataset::XnysBbo, Venue::Xnys) => Ok(Self::XnysBboXnys),
            (Dataset::XnysTrades, Venue::Xnys) => Ok(Self::XnysTradesXnys),
            (Dataset::XnasQbbo, Venue::Xnas) => Ok(Self::XnasQbboXnas),
            (Dataset::XnasNls, Venue::Xnas) => Ok(Self::XnasNlsXnas),
            (Dataset::DbeqPlus, Venue::Xchi) => Ok(Self::DbeqPlusXchi),
            (Dataset::DbeqPlus, Venue::Xcis) => Ok(Self::DbeqPlusXcis),
            (Dataset::DbeqPlus, Venue::Iexg) => Ok(Self::DbeqPlusIexg),
            (Dataset::DbeqPlus, Venue::Eprl) => Ok(Self::DbeqPlusEprl),
            (Dataset::DbeqPlus, Venue::Xnas) => Ok(Self::DbeqPlusXnas),
            (Dataset::DbeqPlus, Venue::Xnys) => Ok(Self::DbeqPlusXnys),
            (Dataset::DbeqPlus, Venue::Finn) => Ok(Self::DbeqPlusFinn),
            (Dataset::DbeqPlus, Venue::Finy) => Ok(Self::DbeqPlusFiny),
            (Dataset::DbeqPlus, Venue::Finc) => Ok(Self::DbeqPlusFinc),
            (Dataset::IfeuImpact, Venue::Ifeu) => Ok(Self::IfeuImpactIfeu),
            (Dataset::NdexImpact, Venue::Ndex) => Ok(Self::NdexImpactNdex),
            (Dataset::DbeqBasic, Venue::Dbeq) => Ok(Self::DbeqBasicDbeq),
            (Dataset::DbeqPlus, Venue::Dbeq) => Ok(Self::DbeqPlusDbeq),
            _ => Err(Error::conversion::<Self>(format!("({dataset}, {venue})"))),
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
            "ARCX.PILLAR.ARCX" => Ok(Self::ArcxPillarArcx),
            "XNYS.BBO.XNYS" => Ok(Self::XnysBboXnys),
            "XNYS.TRADES.XNYS" => Ok(Self::XnysTradesXnys),
            "XNAS.QBBO.XNAS" => Ok(Self::XnasQbboXnas),
            "XNAS.NLS.XNAS" => Ok(Self::XnasNlsXnas),
            "DBEQ.PLUS.XCHI" => Ok(Self::DbeqPlusXchi),
            "DBEQ.PLUS.XCIS" => Ok(Self::DbeqPlusXcis),
            "DBEQ.PLUS.IEXG" => Ok(Self::DbeqPlusIexg),
            "DBEQ.PLUS.EPRL" => Ok(Self::DbeqPlusEprl),
            "DBEQ.PLUS.XNAS" => Ok(Self::DbeqPlusXnas),
            "DBEQ.PLUS.XNYS" => Ok(Self::DbeqPlusXnys),
            "DBEQ.PLUS.FINN" => Ok(Self::DbeqPlusFinn),
            "DBEQ.PLUS.FINY" => Ok(Self::DbeqPlusFiny),
            "DBEQ.PLUS.FINC" => Ok(Self::DbeqPlusFinc),
            "IFEU.IMPACT.IFEU" => Ok(Self::IfeuImpactIfeu),
            "NDEX.IMPACT.NDEX" => Ok(Self::NdexImpactNdex),
            "DBEQ.BASIC.DBEQ" => Ok(Self::DbeqBasicDbeq),
            "DBEQ.PLUS.DBEQ" => Ok(Self::DbeqPlusDbeq),
            _ => Err(Error::conversion::<Self>(s)),
        }
    }
}

#[cfg(feature = "serde")]
mod deserialize {
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer};

    use super::*;

    impl<'de> Deserialize<'de> for Venue {
        fn deserialize<D: Deserializer<'de>>(
            deserializer: D,
        ) -> std::result::Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
        }
    }

    impl<'de> Deserialize<'de> for Dataset {
        fn deserialize<D: Deserializer<'de>>(
            deserializer: D,
        ) -> std::result::Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
        }
    }

    impl<'de> Deserialize<'de> for Publisher {
        fn deserialize<D: Deserializer<'de>>(
            deserializer: D,
        ) -> std::result::Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
        }
    }
}
