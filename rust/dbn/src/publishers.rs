//! Enumerations for different data sources, venues, and publishers.

use std::fmt::{self, Display, Formatter};

use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{Error, Result};

/// A trading execution venue.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, IntoPrimitive, TryFromPrimitive,
)]
#[non_exhaustive]
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
    /// NYSE Texas, Inc.
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
    /// BOX Options
    Xbox = 21,
    /// Cboe Options
    Xcbo = 22,
    /// MIAX Emerald
    Emld = 23,
    /// Cboe EDGX Options
    Edgo = 24,
    /// Nasdaq GEMX
    Gmni = 25,
    /// Nasdaq ISE
    Xisx = 26,
    /// Nasdaq MRX
    Mcry = 27,
    /// MIAX Options
    Xmio = 28,
    /// NYSE Arca Options
    Arco = 29,
    /// Options Price Reporting Authority
    Opra = 30,
    /// MIAX Pearl
    Mprl = 31,
    /// Nasdaq Options
    Xndq = 32,
    /// Nasdaq BX Options
    Xbxo = 33,
    /// Cboe C2 Options
    C2Ox = 34,
    /// Nasdaq PHLX
    Xphl = 35,
    /// Cboe BZX Options
    Bato = 36,
    /// MEMX Options
    Mxop = 37,
    /// ICE Futures Europe (Commodities)
    Ifeu = 38,
    /// ICE Endex
    Ndex = 39,
    /// Databento US Equities - Consolidated
    Dbeq = 40,
    /// MIAX Sapphire
    Sphr = 41,
    /// Long-Term Stock Exchange, Inc.
    Ltse = 42,
    /// Off-Exchange Transactions - Listed Instruments
    Xoff = 43,
    /// IntelligentCross ASPEN Intelligent Bid/Offer
    Aspn = 44,
    /// IntelligentCross ASPEN Maker/Taker
    Asmt = 45,
    /// IntelligentCross ASPEN Inverted
    Aspi = 46,
    /// Databento US Equities - Consolidated
    Equs = 47,
    /// ICE Futures US
    Ifus = 48,
    /// ICE Futures Europe (Financials)
    Ifll = 49,
    /// Eurex Exchange
    Xeur = 50,
    /// European Energy Exchange
    Xeer = 51,
}

/// The number of Venue variants.
pub const VENUE_COUNT: usize = 51;

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
            Self::Sphr => "SPHR",
            Self::Ltse => "LTSE",
            Self::Xoff => "XOFF",
            Self::Aspn => "ASPN",
            Self::Asmt => "ASMT",
            Self::Aspi => "ASPI",
            Self::Equs => "EQUS",
            Self::Ifus => "IFUS",
            Self::Ifll => "IFLL",
            Self::Xeur => "XEUR",
            Self::Xeer => "XEER",
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
            "SPHR" => Ok(Self::Sphr),
            "LTSE" => Ok(Self::Ltse),
            "XOFF" => Ok(Self::Xoff),
            "ASPN" => Ok(Self::Aspn),
            "ASMT" => Ok(Self::Asmt),
            "ASPI" => Ok(Self::Aspi),
            "EQUS" => Ok(Self::Equs),
            "IFUS" => Ok(Self::Ifus),
            "IFLL" => Ok(Self::Ifll),
            "XEUR" => Ok(Self::Xeur),
            "XEER" => Ok(Self::Xeer),
            _ => Err(Error::conversion::<Self>(s)),
        }
    }
}

/// A source of data.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, IntoPrimitive, TryFromPrimitive,
)]
#[non_exhaustive]
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
    /// Cboe BZX Depth
    BatsPitch = 5,
    /// Cboe BYX Depth
    BatyPitch = 6,
    /// Cboe EDGA Depth
    EdgaPitch = 7,
    /// Cboe EDGX Depth
    EdgxPitch = 8,
    /// NYSE Integrated
    XnysPillar = 9,
    /// NYSE National Integrated
    XcisPillar = 10,
    /// NYSE American Integrated
    XasePillar = 11,
    /// NYSE Texas Integrated
    XchiPillar = 12,
    /// NYSE National BBO
    XcisBbo = 13,
    /// NYSE National Trades
    XcisTrades = 14,
    /// MEMX Memoir Depth
    MemxMemoir = 15,
    /// MIAX Pearl Depth
    EprlDom = 16,
    /// FINRA/Nasdaq TRF (DEPRECATED)
    #[deprecated(since = "0.17.0")]
    FinnNls = 17,
    /// FINRA/NYSE TRF (DEPRECATED)
    #[deprecated(since = "0.17.0")]
    FinyTrades = 18,
    /// OPRA Binary
    OpraPillar = 19,
    /// Databento US Equities Basic
    DbeqBasic = 20,
    /// NYSE Arca Integrated
    ArcxPillar = 21,
    /// IEX TOPS
    IexgTops = 22,
    /// Databento US Equities Plus
    EqusPlus = 23,
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
    /// Databento US Equities (All Feeds)
    EqusAll = 30,
    /// Nasdaq Basic (NLS and QBBO)
    XnasBasic = 31,
    /// Databento US Equities Summary
    EqusSummary = 32,
    /// NYSE National Trades and BBO
    XcisTradesbbo = 33,
    /// NYSE Trades and BBO
    XnysTradesbbo = 34,
    /// Databento US Equities Mini
    EqusMini = 35,
    /// ICE Futures US iMpact
    IfusImpact = 36,
    /// ICE Futures Europe (Financials) iMpact
    IfllImpact = 37,
    /// Eurex EOBI
    XeurEobi = 38,
    /// European Energy Exchange EOBI
    XeerEobi = 39,
}

/// The number of Dataset variants.
pub const DATASET_COUNT: usize = 39;

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
            #[allow(deprecated)]
            Self::FinnNls => "FINN.NLS",
            #[allow(deprecated)]
            Self::FinyTrades => "FINY.TRADES",
            Self::OpraPillar => "OPRA.PILLAR",
            Self::DbeqBasic => "DBEQ.BASIC",
            Self::ArcxPillar => "ARCX.PILLAR",
            Self::IexgTops => "IEXG.TOPS",
            Self::EqusPlus => "EQUS.PLUS",
            Self::XnysBbo => "XNYS.BBO",
            Self::XnysTrades => "XNYS.TRADES",
            Self::XnasQbbo => "XNAS.QBBO",
            Self::XnasNls => "XNAS.NLS",
            Self::IfeuImpact => "IFEU.IMPACT",
            Self::NdexImpact => "NDEX.IMPACT",
            Self::EqusAll => "EQUS.ALL",
            Self::XnasBasic => "XNAS.BASIC",
            Self::EqusSummary => "EQUS.SUMMARY",
            Self::XcisTradesbbo => "XCIS.TRADESBBO",
            Self::XnysTradesbbo => "XNYS.TRADESBBO",
            Self::EqusMini => "EQUS.MINI",
            Self::IfusImpact => "IFUS.IMPACT",
            Self::IfllImpact => "IFLL.IMPACT",
            Self::XeurEobi => "XEUR.EOBI",
            Self::XeerEobi => "XEER.EOBI",
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
            #[allow(deprecated)]
            "FINN.NLS" => Ok(Self::FinnNls),
            #[allow(deprecated)]
            "FINY.TRADES" => Ok(Self::FinyTrades),
            "OPRA.PILLAR" => Ok(Self::OpraPillar),
            "DBEQ.BASIC" => Ok(Self::DbeqBasic),
            "ARCX.PILLAR" => Ok(Self::ArcxPillar),
            "IEXG.TOPS" => Ok(Self::IexgTops),
            "EQUS.PLUS" => Ok(Self::EqusPlus),
            "XNYS.BBO" => Ok(Self::XnysBbo),
            "XNYS.TRADES" => Ok(Self::XnysTrades),
            "XNAS.QBBO" => Ok(Self::XnasQbbo),
            "XNAS.NLS" => Ok(Self::XnasNls),
            "IFEU.IMPACT" => Ok(Self::IfeuImpact),
            "NDEX.IMPACT" => Ok(Self::NdexImpact),
            "EQUS.ALL" => Ok(Self::EqusAll),
            "XNAS.BASIC" => Ok(Self::XnasBasic),
            "EQUS.SUMMARY" => Ok(Self::EqusSummary),
            "XCIS.TRADESBBO" => Ok(Self::XcisTradesbbo),
            "XNYS.TRADESBBO" => Ok(Self::XnysTradesbbo),
            "EQUS.MINI" => Ok(Self::EqusMini),
            "IFUS.IMPACT" => Ok(Self::IfusImpact),
            "IFLL.IMPACT" => Ok(Self::IfllImpact),
            "XEUR.EOBI" => Ok(Self::XeurEobi),
            "XEER.EOBI" => Ok(Self::XeerEobi),
            _ => Err(Error::conversion::<Self>(s)),
        }
    }
}

/// A specific Venue from a specific data source.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, IntoPrimitive, TryFromPrimitive,
)]
#[non_exhaustive]
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
    /// Cboe BZX Depth
    BatsPitchBats = 5,
    /// Cboe BYX Depth
    BatyPitchBaty = 6,
    /// Cboe EDGA Depth
    EdgaPitchEdga = 7,
    /// Cboe EDGX Depth
    EdgxPitchEdgx = 8,
    /// NYSE Integrated
    XnysPillarXnys = 9,
    /// NYSE National Integrated
    XcisPillarXcis = 10,
    /// NYSE American Integrated
    XasePillarXase = 11,
    /// NYSE Texas Integrated
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
    XnasNlsFinn = 17,
    /// FINRA/Nasdaq TRF Chicago
    XnasNlsFinc = 18,
    /// FINRA/NYSE TRF
    XnysTradesFiny = 19,
    /// OPRA - NYSE American Options
    OpraPillarAmxo = 20,
    /// OPRA - BOX Options
    OpraPillarXbox = 21,
    /// OPRA - Cboe Options
    OpraPillarXcbo = 22,
    /// OPRA - MIAX Emerald
    OpraPillarEmld = 23,
    /// OPRA - Cboe EDGX Options
    OpraPillarEdgo = 24,
    /// OPRA - Nasdaq GEMX
    OpraPillarGmni = 25,
    /// OPRA - Nasdaq ISE
    OpraPillarXisx = 26,
    /// OPRA - Nasdaq MRX
    OpraPillarMcry = 27,
    /// OPRA - MIAX Options
    OpraPillarXmio = 28,
    /// OPRA - NYSE Arca Options
    OpraPillarArco = 29,
    /// OPRA - Options Price Reporting Authority
    OpraPillarOpra = 30,
    /// OPRA - MIAX Pearl
    OpraPillarMprl = 31,
    /// OPRA - Nasdaq Options
    OpraPillarXndq = 32,
    /// OPRA - Nasdaq BX Options
    OpraPillarXbxo = 33,
    /// OPRA - Cboe C2 Options
    OpraPillarC2Ox = 34,
    /// OPRA - Nasdaq PHLX
    OpraPillarXphl = 35,
    /// OPRA - Cboe BZX Options
    OpraPillarBato = 36,
    /// OPRA - MEMX Options
    OpraPillarMxop = 37,
    /// IEX TOPS
    IexgTopsIexg = 38,
    /// DBEQ Basic - NYSE Texas
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
    /// Databento US Equities Plus - NYSE Texas
    EqusPlusXchi = 48,
    /// Databento US Equities Plus - NYSE National
    EqusPlusXcis = 49,
    /// Databento US Equities Plus - IEX
    EqusPlusIexg = 50,
    /// Databento US Equities Plus - MIAX Pearl
    EqusPlusEprl = 51,
    /// Databento US Equities Plus - Nasdaq
    EqusPlusXnas = 52,
    /// Databento US Equities Plus - NYSE
    EqusPlusXnys = 53,
    /// Databento US Equities Plus - FINRA/Nasdaq TRF Carteret
    EqusPlusFinn = 54,
    /// Databento US Equities Plus - FINRA/NYSE TRF
    EqusPlusFiny = 55,
    /// Databento US Equities Plus - FINRA/Nasdaq TRF Chicago
    EqusPlusFinc = 56,
    /// ICE Futures Europe (Commodities)
    IfeuImpactIfeu = 57,
    /// ICE Endex
    NdexImpactNdex = 58,
    /// Databento US Equities Basic - Consolidated
    DbeqBasicDbeq = 59,
    /// EQUS Plus - Consolidated
    EqusPlusEqus = 60,
    /// OPRA - MIAX Sapphire
    OpraPillarSphr = 61,
    /// Databento US Equities (All Feeds) - NYSE Texas
    EqusAllXchi = 62,
    /// Databento US Equities (All Feeds) - NYSE National
    EqusAllXcis = 63,
    /// Databento US Equities (All Feeds) - IEX
    EqusAllIexg = 64,
    /// Databento US Equities (All Feeds) - MIAX Pearl
    EqusAllEprl = 65,
    /// Databento US Equities (All Feeds) - Nasdaq
    EqusAllXnas = 66,
    /// Databento US Equities (All Feeds) - NYSE
    EqusAllXnys = 67,
    /// Databento US Equities (All Feeds) - FINRA/Nasdaq TRF Carteret
    EqusAllFinn = 68,
    /// Databento US Equities (All Feeds) - FINRA/NYSE TRF
    EqusAllFiny = 69,
    /// Databento US Equities (All Feeds) - FINRA/Nasdaq TRF Chicago
    EqusAllFinc = 70,
    /// Databento US Equities (All Feeds) - Cboe BZX
    EqusAllBats = 71,
    /// Databento US Equities (All Feeds) - Cboe BYX
    EqusAllBaty = 72,
    /// Databento US Equities (All Feeds) - Cboe EDGA
    EqusAllEdga = 73,
    /// Databento US Equities (All Feeds) - Cboe EDGX
    EqusAllEdgx = 74,
    /// Databento US Equities (All Feeds) - Nasdaq BX
    EqusAllXbos = 75,
    /// Databento US Equities (All Feeds) - Nasdaq PSX
    EqusAllXpsx = 76,
    /// Databento US Equities (All Feeds) - MEMX
    EqusAllMemx = 77,
    /// Databento US Equities (All Feeds) - NYSE American
    EqusAllXase = 78,
    /// Databento US Equities (All Feeds) - NYSE Arca
    EqusAllArcx = 79,
    /// Databento US Equities (All Feeds) - Long-Term Stock Exchange
    EqusAllLtse = 80,
    /// Nasdaq Basic - Nasdaq
    XnasBasicXnas = 81,
    /// Nasdaq Basic - FINRA/Nasdaq TRF Carteret
    XnasBasicFinn = 82,
    /// Nasdaq Basic - FINRA/Nasdaq TRF Chicago
    XnasBasicFinc = 83,
    /// ICE Futures Europe - Off-Market Trades
    IfeuImpactXoff = 84,
    /// ICE Endex - Off-Market Trades
    NdexImpactXoff = 85,
    /// Nasdaq NLS - Nasdaq BX
    XnasNlsXbos = 86,
    /// Nasdaq NLS - Nasdaq PSX
    XnasNlsXpsx = 87,
    /// Nasdaq Basic - Nasdaq BX
    XnasBasicXbos = 88,
    /// Nasdaq Basic - Nasdaq PSX
    XnasBasicXpsx = 89,
    /// Databento Equities Summary
    EqusSummaryEqus = 90,
    /// NYSE National Trades and BBO
    XcisTradesbboXcis = 91,
    /// NYSE Trades and BBO
    XnysTradesbboXnys = 92,
    /// Nasdaq Basic - Consolidated
    XnasBasicEqus = 93,
    /// Databento US Equities (All Feeds) - Consolidated
    EqusAllEqus = 94,
    /// Databento US Equities Mini
    EqusMiniEqus = 95,
    /// NYSE Trades - Consolidated
    XnysTradesEqus = 96,
    /// ICE Futures US
    IfusImpactIfus = 97,
    /// ICE Futures US - Off-Market Trades
    IfusImpactXoff = 98,
    /// ICE Futures Europe (Financials)
    IfllImpactIfll = 99,
    /// ICE Futures Europe (Financials) - Off-Market Trades
    IfllImpactXoff = 100,
    /// Eurex EOBI
    XeurEobiXeur = 101,
    /// European Energy Exchange EOBI
    XeerEobiXeer = 102,
    /// Eurex EOBI - Off-Market Trades
    XeurEobiXoff = 103,
    /// European Energy Exchange EOBI - Off-Market Trades
    XeerEobiXoff = 104,
}

/// The number of Publisher variants.
pub const PUBLISHER_COUNT: usize = 104;

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
            Self::XnasNlsFinn => "XNAS.NLS.FINN",
            Self::XnasNlsFinc => "XNAS.NLS.FINC",
            Self::XnysTradesFiny => "XNYS.TRADES.FINY",
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
            Self::EqusPlusXchi => "EQUS.PLUS.XCHI",
            Self::EqusPlusXcis => "EQUS.PLUS.XCIS",
            Self::EqusPlusIexg => "EQUS.PLUS.IEXG",
            Self::EqusPlusEprl => "EQUS.PLUS.EPRL",
            Self::EqusPlusXnas => "EQUS.PLUS.XNAS",
            Self::EqusPlusXnys => "EQUS.PLUS.XNYS",
            Self::EqusPlusFinn => "EQUS.PLUS.FINN",
            Self::EqusPlusFiny => "EQUS.PLUS.FINY",
            Self::EqusPlusFinc => "EQUS.PLUS.FINC",
            Self::IfeuImpactIfeu => "IFEU.IMPACT.IFEU",
            Self::NdexImpactNdex => "NDEX.IMPACT.NDEX",
            Self::DbeqBasicDbeq => "DBEQ.BASIC.DBEQ",
            Self::EqusPlusEqus => "EQUS.PLUS.EQUS",
            Self::OpraPillarSphr => "OPRA.PILLAR.SPHR",
            Self::EqusAllXchi => "EQUS.ALL.XCHI",
            Self::EqusAllXcis => "EQUS.ALL.XCIS",
            Self::EqusAllIexg => "EQUS.ALL.IEXG",
            Self::EqusAllEprl => "EQUS.ALL.EPRL",
            Self::EqusAllXnas => "EQUS.ALL.XNAS",
            Self::EqusAllXnys => "EQUS.ALL.XNYS",
            Self::EqusAllFinn => "EQUS.ALL.FINN",
            Self::EqusAllFiny => "EQUS.ALL.FINY",
            Self::EqusAllFinc => "EQUS.ALL.FINC",
            Self::EqusAllBats => "EQUS.ALL.BATS",
            Self::EqusAllBaty => "EQUS.ALL.BATY",
            Self::EqusAllEdga => "EQUS.ALL.EDGA",
            Self::EqusAllEdgx => "EQUS.ALL.EDGX",
            Self::EqusAllXbos => "EQUS.ALL.XBOS",
            Self::EqusAllXpsx => "EQUS.ALL.XPSX",
            Self::EqusAllMemx => "EQUS.ALL.MEMX",
            Self::EqusAllXase => "EQUS.ALL.XASE",
            Self::EqusAllArcx => "EQUS.ALL.ARCX",
            Self::EqusAllLtse => "EQUS.ALL.LTSE",
            Self::XnasBasicXnas => "XNAS.BASIC.XNAS",
            Self::XnasBasicFinn => "XNAS.BASIC.FINN",
            Self::XnasBasicFinc => "XNAS.BASIC.FINC",
            Self::IfeuImpactXoff => "IFEU.IMPACT.XOFF",
            Self::NdexImpactXoff => "NDEX.IMPACT.XOFF",
            Self::XnasNlsXbos => "XNAS.NLS.XBOS",
            Self::XnasNlsXpsx => "XNAS.NLS.XPSX",
            Self::XnasBasicXbos => "XNAS.BASIC.XBOS",
            Self::XnasBasicXpsx => "XNAS.BASIC.XPSX",
            Self::EqusSummaryEqus => "EQUS.SUMMARY.EQUS",
            Self::XcisTradesbboXcis => "XCIS.TRADESBBO.XCIS",
            Self::XnysTradesbboXnys => "XNYS.TRADESBBO.XNYS",
            Self::XnasBasicEqus => "XNAS.BASIC.EQUS",
            Self::EqusAllEqus => "EQUS.ALL.EQUS",
            Self::EqusMiniEqus => "EQUS.MINI.EQUS",
            Self::XnysTradesEqus => "XNYS.TRADES.EQUS",
            Self::IfusImpactIfus => "IFUS.IMPACT.IFUS",
            Self::IfusImpactXoff => "IFUS.IMPACT.XOFF",
            Self::IfllImpactIfll => "IFLL.IMPACT.IFLL",
            Self::IfllImpactXoff => "IFLL.IMPACT.XOFF",
            Self::XeurEobiXeur => "XEUR.EOBI.XEUR",
            Self::XeerEobiXeer => "XEER.EOBI.XEER",
            Self::XeurEobiXoff => "XEUR.EOBI.XOFF",
            Self::XeerEobiXoff => "XEER.EOBI.XOFF",
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
            Self::XnasNlsFinn => Venue::Finn,
            Self::XnasNlsFinc => Venue::Finc,
            Self::XnysTradesFiny => Venue::Finy,
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
            Self::EqusPlusXchi => Venue::Xchi,
            Self::EqusPlusXcis => Venue::Xcis,
            Self::EqusPlusIexg => Venue::Iexg,
            Self::EqusPlusEprl => Venue::Eprl,
            Self::EqusPlusXnas => Venue::Xnas,
            Self::EqusPlusXnys => Venue::Xnys,
            Self::EqusPlusFinn => Venue::Finn,
            Self::EqusPlusFiny => Venue::Finy,
            Self::EqusPlusFinc => Venue::Finc,
            Self::IfeuImpactIfeu => Venue::Ifeu,
            Self::NdexImpactNdex => Venue::Ndex,
            Self::DbeqBasicDbeq => Venue::Dbeq,
            Self::EqusPlusEqus => Venue::Equs,
            Self::OpraPillarSphr => Venue::Sphr,
            Self::EqusAllXchi => Venue::Xchi,
            Self::EqusAllXcis => Venue::Xcis,
            Self::EqusAllIexg => Venue::Iexg,
            Self::EqusAllEprl => Venue::Eprl,
            Self::EqusAllXnas => Venue::Xnas,
            Self::EqusAllXnys => Venue::Xnys,
            Self::EqusAllFinn => Venue::Finn,
            Self::EqusAllFiny => Venue::Finy,
            Self::EqusAllFinc => Venue::Finc,
            Self::EqusAllBats => Venue::Bats,
            Self::EqusAllBaty => Venue::Baty,
            Self::EqusAllEdga => Venue::Edga,
            Self::EqusAllEdgx => Venue::Edgx,
            Self::EqusAllXbos => Venue::Xbos,
            Self::EqusAllXpsx => Venue::Xpsx,
            Self::EqusAllMemx => Venue::Memx,
            Self::EqusAllXase => Venue::Xase,
            Self::EqusAllArcx => Venue::Arcx,
            Self::EqusAllLtse => Venue::Ltse,
            Self::XnasBasicXnas => Venue::Xnas,
            Self::XnasBasicFinn => Venue::Finn,
            Self::XnasBasicFinc => Venue::Finc,
            Self::IfeuImpactXoff => Venue::Xoff,
            Self::NdexImpactXoff => Venue::Xoff,
            Self::XnasNlsXbos => Venue::Xbos,
            Self::XnasNlsXpsx => Venue::Xpsx,
            Self::XnasBasicXbos => Venue::Xbos,
            Self::XnasBasicXpsx => Venue::Xpsx,
            Self::EqusSummaryEqus => Venue::Equs,
            Self::XcisTradesbboXcis => Venue::Xcis,
            Self::XnysTradesbboXnys => Venue::Xnys,
            Self::XnasBasicEqus => Venue::Equs,
            Self::EqusAllEqus => Venue::Equs,
            Self::EqusMiniEqus => Venue::Equs,
            Self::XnysTradesEqus => Venue::Equs,
            Self::IfusImpactIfus => Venue::Ifus,
            Self::IfusImpactXoff => Venue::Xoff,
            Self::IfllImpactIfll => Venue::Ifll,
            Self::IfllImpactXoff => Venue::Xoff,
            Self::XeurEobiXeur => Venue::Xeur,
            Self::XeerEobiXeer => Venue::Xeer,
            Self::XeurEobiXoff => Venue::Xoff,
            Self::XeerEobiXoff => Venue::Xoff,
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
            Self::XnasNlsFinn => Dataset::XnasNls,
            Self::XnasNlsFinc => Dataset::XnasNls,
            Self::XnysTradesFiny => Dataset::XnysTrades,
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
            Self::EqusPlusXchi => Dataset::EqusPlus,
            Self::EqusPlusXcis => Dataset::EqusPlus,
            Self::EqusPlusIexg => Dataset::EqusPlus,
            Self::EqusPlusEprl => Dataset::EqusPlus,
            Self::EqusPlusXnas => Dataset::EqusPlus,
            Self::EqusPlusXnys => Dataset::EqusPlus,
            Self::EqusPlusFinn => Dataset::EqusPlus,
            Self::EqusPlusFiny => Dataset::EqusPlus,
            Self::EqusPlusFinc => Dataset::EqusPlus,
            Self::IfeuImpactIfeu => Dataset::IfeuImpact,
            Self::NdexImpactNdex => Dataset::NdexImpact,
            Self::DbeqBasicDbeq => Dataset::DbeqBasic,
            Self::EqusPlusEqus => Dataset::EqusPlus,
            Self::OpraPillarSphr => Dataset::OpraPillar,
            Self::EqusAllXchi => Dataset::EqusAll,
            Self::EqusAllXcis => Dataset::EqusAll,
            Self::EqusAllIexg => Dataset::EqusAll,
            Self::EqusAllEprl => Dataset::EqusAll,
            Self::EqusAllXnas => Dataset::EqusAll,
            Self::EqusAllXnys => Dataset::EqusAll,
            Self::EqusAllFinn => Dataset::EqusAll,
            Self::EqusAllFiny => Dataset::EqusAll,
            Self::EqusAllFinc => Dataset::EqusAll,
            Self::EqusAllBats => Dataset::EqusAll,
            Self::EqusAllBaty => Dataset::EqusAll,
            Self::EqusAllEdga => Dataset::EqusAll,
            Self::EqusAllEdgx => Dataset::EqusAll,
            Self::EqusAllXbos => Dataset::EqusAll,
            Self::EqusAllXpsx => Dataset::EqusAll,
            Self::EqusAllMemx => Dataset::EqusAll,
            Self::EqusAllXase => Dataset::EqusAll,
            Self::EqusAllArcx => Dataset::EqusAll,
            Self::EqusAllLtse => Dataset::EqusAll,
            Self::XnasBasicXnas => Dataset::XnasBasic,
            Self::XnasBasicFinn => Dataset::XnasBasic,
            Self::XnasBasicFinc => Dataset::XnasBasic,
            Self::IfeuImpactXoff => Dataset::IfeuImpact,
            Self::NdexImpactXoff => Dataset::NdexImpact,
            Self::XnasNlsXbos => Dataset::XnasNls,
            Self::XnasNlsXpsx => Dataset::XnasNls,
            Self::XnasBasicXbos => Dataset::XnasBasic,
            Self::XnasBasicXpsx => Dataset::XnasBasic,
            Self::EqusSummaryEqus => Dataset::EqusSummary,
            Self::XcisTradesbboXcis => Dataset::XcisTradesbbo,
            Self::XnysTradesbboXnys => Dataset::XnysTradesbbo,
            Self::XnasBasicEqus => Dataset::XnasBasic,
            Self::EqusAllEqus => Dataset::EqusAll,
            Self::EqusMiniEqus => Dataset::EqusMini,
            Self::XnysTradesEqus => Dataset::XnysTrades,
            Self::IfusImpactIfus => Dataset::IfusImpact,
            Self::IfusImpactXoff => Dataset::IfusImpact,
            Self::IfllImpactIfll => Dataset::IfllImpact,
            Self::IfllImpactXoff => Dataset::IfllImpact,
            Self::XeurEobiXeur => Dataset::XeurEobi,
            Self::XeerEobiXeer => Dataset::XeerEobi,
            Self::XeurEobiXoff => Dataset::XeurEobi,
            Self::XeerEobiXoff => Dataset::XeerEobi,
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
            (Dataset::XnasNls, Venue::Finn) => Ok(Self::XnasNlsFinn),
            (Dataset::XnasNls, Venue::Finc) => Ok(Self::XnasNlsFinc),
            (Dataset::XnysTrades, Venue::Finy) => Ok(Self::XnysTradesFiny),
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
            (Dataset::EqusPlus, Venue::Xchi) => Ok(Self::EqusPlusXchi),
            (Dataset::EqusPlus, Venue::Xcis) => Ok(Self::EqusPlusXcis),
            (Dataset::EqusPlus, Venue::Iexg) => Ok(Self::EqusPlusIexg),
            (Dataset::EqusPlus, Venue::Eprl) => Ok(Self::EqusPlusEprl),
            (Dataset::EqusPlus, Venue::Xnas) => Ok(Self::EqusPlusXnas),
            (Dataset::EqusPlus, Venue::Xnys) => Ok(Self::EqusPlusXnys),
            (Dataset::EqusPlus, Venue::Finn) => Ok(Self::EqusPlusFinn),
            (Dataset::EqusPlus, Venue::Finy) => Ok(Self::EqusPlusFiny),
            (Dataset::EqusPlus, Venue::Finc) => Ok(Self::EqusPlusFinc),
            (Dataset::IfeuImpact, Venue::Ifeu) => Ok(Self::IfeuImpactIfeu),
            (Dataset::NdexImpact, Venue::Ndex) => Ok(Self::NdexImpactNdex),
            (Dataset::DbeqBasic, Venue::Dbeq) => Ok(Self::DbeqBasicDbeq),
            (Dataset::EqusPlus, Venue::Equs) => Ok(Self::EqusPlusEqus),
            (Dataset::OpraPillar, Venue::Sphr) => Ok(Self::OpraPillarSphr),
            (Dataset::EqusAll, Venue::Xchi) => Ok(Self::EqusAllXchi),
            (Dataset::EqusAll, Venue::Xcis) => Ok(Self::EqusAllXcis),
            (Dataset::EqusAll, Venue::Iexg) => Ok(Self::EqusAllIexg),
            (Dataset::EqusAll, Venue::Eprl) => Ok(Self::EqusAllEprl),
            (Dataset::EqusAll, Venue::Xnas) => Ok(Self::EqusAllXnas),
            (Dataset::EqusAll, Venue::Xnys) => Ok(Self::EqusAllXnys),
            (Dataset::EqusAll, Venue::Finn) => Ok(Self::EqusAllFinn),
            (Dataset::EqusAll, Venue::Finy) => Ok(Self::EqusAllFiny),
            (Dataset::EqusAll, Venue::Finc) => Ok(Self::EqusAllFinc),
            (Dataset::EqusAll, Venue::Bats) => Ok(Self::EqusAllBats),
            (Dataset::EqusAll, Venue::Baty) => Ok(Self::EqusAllBaty),
            (Dataset::EqusAll, Venue::Edga) => Ok(Self::EqusAllEdga),
            (Dataset::EqusAll, Venue::Edgx) => Ok(Self::EqusAllEdgx),
            (Dataset::EqusAll, Venue::Xbos) => Ok(Self::EqusAllXbos),
            (Dataset::EqusAll, Venue::Xpsx) => Ok(Self::EqusAllXpsx),
            (Dataset::EqusAll, Venue::Memx) => Ok(Self::EqusAllMemx),
            (Dataset::EqusAll, Venue::Xase) => Ok(Self::EqusAllXase),
            (Dataset::EqusAll, Venue::Arcx) => Ok(Self::EqusAllArcx),
            (Dataset::EqusAll, Venue::Ltse) => Ok(Self::EqusAllLtse),
            (Dataset::XnasBasic, Venue::Xnas) => Ok(Self::XnasBasicXnas),
            (Dataset::XnasBasic, Venue::Finn) => Ok(Self::XnasBasicFinn),
            (Dataset::XnasBasic, Venue::Finc) => Ok(Self::XnasBasicFinc),
            (Dataset::IfeuImpact, Venue::Xoff) => Ok(Self::IfeuImpactXoff),
            (Dataset::NdexImpact, Venue::Xoff) => Ok(Self::NdexImpactXoff),
            (Dataset::XnasNls, Venue::Xbos) => Ok(Self::XnasNlsXbos),
            (Dataset::XnasNls, Venue::Xpsx) => Ok(Self::XnasNlsXpsx),
            (Dataset::XnasBasic, Venue::Xbos) => Ok(Self::XnasBasicXbos),
            (Dataset::XnasBasic, Venue::Xpsx) => Ok(Self::XnasBasicXpsx),
            (Dataset::EqusSummary, Venue::Equs) => Ok(Self::EqusSummaryEqus),
            (Dataset::XcisTradesbbo, Venue::Xcis) => Ok(Self::XcisTradesbboXcis),
            (Dataset::XnysTradesbbo, Venue::Xnys) => Ok(Self::XnysTradesbboXnys),
            (Dataset::XnasBasic, Venue::Equs) => Ok(Self::XnasBasicEqus),
            (Dataset::EqusAll, Venue::Equs) => Ok(Self::EqusAllEqus),
            (Dataset::EqusMini, Venue::Equs) => Ok(Self::EqusMiniEqus),
            (Dataset::XnysTrades, Venue::Equs) => Ok(Self::XnysTradesEqus),
            (Dataset::IfusImpact, Venue::Ifus) => Ok(Self::IfusImpactIfus),
            (Dataset::IfusImpact, Venue::Xoff) => Ok(Self::IfusImpactXoff),
            (Dataset::IfllImpact, Venue::Ifll) => Ok(Self::IfllImpactIfll),
            (Dataset::IfllImpact, Venue::Xoff) => Ok(Self::IfllImpactXoff),
            (Dataset::XeurEobi, Venue::Xeur) => Ok(Self::XeurEobiXeur),
            (Dataset::XeerEobi, Venue::Xeer) => Ok(Self::XeerEobiXeer),
            (Dataset::XeurEobi, Venue::Xoff) => Ok(Self::XeurEobiXoff),
            (Dataset::XeerEobi, Venue::Xoff) => Ok(Self::XeerEobiXoff),
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
            "XNAS.NLS.FINN" => Ok(Self::XnasNlsFinn),
            "XNAS.NLS.FINC" => Ok(Self::XnasNlsFinc),
            "XNYS.TRADES.FINY" => Ok(Self::XnysTradesFiny),
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
            "EQUS.PLUS.XCHI" => Ok(Self::EqusPlusXchi),
            "EQUS.PLUS.XCIS" => Ok(Self::EqusPlusXcis),
            "EQUS.PLUS.IEXG" => Ok(Self::EqusPlusIexg),
            "EQUS.PLUS.EPRL" => Ok(Self::EqusPlusEprl),
            "EQUS.PLUS.XNAS" => Ok(Self::EqusPlusXnas),
            "EQUS.PLUS.XNYS" => Ok(Self::EqusPlusXnys),
            "EQUS.PLUS.FINN" => Ok(Self::EqusPlusFinn),
            "EQUS.PLUS.FINY" => Ok(Self::EqusPlusFiny),
            "EQUS.PLUS.FINC" => Ok(Self::EqusPlusFinc),
            "IFEU.IMPACT.IFEU" => Ok(Self::IfeuImpactIfeu),
            "NDEX.IMPACT.NDEX" => Ok(Self::NdexImpactNdex),
            "DBEQ.BASIC.DBEQ" => Ok(Self::DbeqBasicDbeq),
            "EQUS.PLUS.EQUS" => Ok(Self::EqusPlusEqus),
            "OPRA.PILLAR.SPHR" => Ok(Self::OpraPillarSphr),
            "EQUS.ALL.XCHI" => Ok(Self::EqusAllXchi),
            "EQUS.ALL.XCIS" => Ok(Self::EqusAllXcis),
            "EQUS.ALL.IEXG" => Ok(Self::EqusAllIexg),
            "EQUS.ALL.EPRL" => Ok(Self::EqusAllEprl),
            "EQUS.ALL.XNAS" => Ok(Self::EqusAllXnas),
            "EQUS.ALL.XNYS" => Ok(Self::EqusAllXnys),
            "EQUS.ALL.FINN" => Ok(Self::EqusAllFinn),
            "EQUS.ALL.FINY" => Ok(Self::EqusAllFiny),
            "EQUS.ALL.FINC" => Ok(Self::EqusAllFinc),
            "EQUS.ALL.BATS" => Ok(Self::EqusAllBats),
            "EQUS.ALL.BATY" => Ok(Self::EqusAllBaty),
            "EQUS.ALL.EDGA" => Ok(Self::EqusAllEdga),
            "EQUS.ALL.EDGX" => Ok(Self::EqusAllEdgx),
            "EQUS.ALL.XBOS" => Ok(Self::EqusAllXbos),
            "EQUS.ALL.XPSX" => Ok(Self::EqusAllXpsx),
            "EQUS.ALL.MEMX" => Ok(Self::EqusAllMemx),
            "EQUS.ALL.XASE" => Ok(Self::EqusAllXase),
            "EQUS.ALL.ARCX" => Ok(Self::EqusAllArcx),
            "EQUS.ALL.LTSE" => Ok(Self::EqusAllLtse),
            "XNAS.BASIC.XNAS" => Ok(Self::XnasBasicXnas),
            "XNAS.BASIC.FINN" => Ok(Self::XnasBasicFinn),
            "XNAS.BASIC.FINC" => Ok(Self::XnasBasicFinc),
            "IFEU.IMPACT.XOFF" => Ok(Self::IfeuImpactXoff),
            "NDEX.IMPACT.XOFF" => Ok(Self::NdexImpactXoff),
            "XNAS.NLS.XBOS" => Ok(Self::XnasNlsXbos),
            "XNAS.NLS.XPSX" => Ok(Self::XnasNlsXpsx),
            "XNAS.BASIC.XBOS" => Ok(Self::XnasBasicXbos),
            "XNAS.BASIC.XPSX" => Ok(Self::XnasBasicXpsx),
            "EQUS.SUMMARY.EQUS" => Ok(Self::EqusSummaryEqus),
            "XCIS.TRADESBBO.XCIS" => Ok(Self::XcisTradesbboXcis),
            "XNYS.TRADESBBO.XNYS" => Ok(Self::XnysTradesbboXnys),
            "XNAS.BASIC.EQUS" => Ok(Self::XnasBasicEqus),
            "EQUS.ALL.EQUS" => Ok(Self::EqusAllEqus),
            "EQUS.MINI.EQUS" => Ok(Self::EqusMiniEqus),
            "XNYS.TRADES.EQUS" => Ok(Self::XnysTradesEqus),
            "IFUS.IMPACT.IFUS" => Ok(Self::IfusImpactIfus),
            "IFUS.IMPACT.XOFF" => Ok(Self::IfusImpactXoff),
            "IFLL.IMPACT.IFLL" => Ok(Self::IfllImpactIfll),
            "IFLL.IMPACT.XOFF" => Ok(Self::IfllImpactXoff),
            "XEUR.EOBI.XEUR" => Ok(Self::XeurEobiXeur),
            "XEER.EOBI.XEER" => Ok(Self::XeerEobiXeer),
            "XEUR.EOBI.XOFF" => Ok(Self::XeurEobiXoff),
            "XEER.EOBI.XOFF" => Ok(Self::XeerEobiXoff),
            _ => Err(Error::conversion::<Self>(s)),
        }
    }
}

#[cfg(feature = "serde")]
mod deserialize {
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer, Serialize};

    use super::*;

    impl<'de> Deserialize<'de> for Venue {
        fn deserialize<D: Deserializer<'de>>(
            deserializer: D,
        ) -> std::result::Result<Self, D::Error> {
            let str = String::deserialize(deserializer)?;
            FromStr::from_str(&str).map_err(de::Error::custom)
        }
    }

    impl Serialize for Venue {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.as_str().serialize(serializer)
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

    impl Serialize for Dataset {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.as_str().serialize(serializer)
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

    impl Serialize for Publisher {
        fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.as_str().serialize(serializer)
        }
    }
}
