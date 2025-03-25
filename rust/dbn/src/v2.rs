//! Record data types for encoding different Databento [`Schema`](crate::enums::Schema)s
//! in DBN version 2.

use std::os::raw::c_char;

use crate::compat::InstrumentDefRec;
pub use crate::compat::SYMBOL_CSTR_LEN_V2 as SYMBOL_CSTR_LEN;
pub use crate::record::{
    Bbo1MMsg, Bbo1SMsg, BboMsg, Cbbo1MMsg, Cbbo1SMsg, CbboMsg, Cmbp1Msg, ErrorMsg, ImbalanceMsg,
    InstrumentDefMsg, MboMsg, OhlcvMsg, StatMsg, StatusMsg, SymbolMappingMsg, SystemMsg, TbboMsg,
    TcbboMsg, TradeMsg, WithTsOut,
};

use crate::{compat::SymbolMappingRec, rtype, v1, RecordHeader};

impl From<&v1::InstrumentDefMsg> for InstrumentDefMsg {
    fn from(old: &v1::InstrumentDefMsg) -> Self {
        let mut res = Self {
            // recalculate length
            hd: RecordHeader::new::<Self>(
                rtype::INSTRUMENT_DEF,
                old.hd.publisher_id,
                old.hd.instrument_id,
                old.hd.ts_event,
            ),
            ts_recv: old.ts_recv,
            min_price_increment: old.min_price_increment,
            display_factor: old.display_factor,
            expiration: old.expiration,
            activation: old.activation,
            high_limit_price: old.high_limit_price,
            low_limit_price: old.low_limit_price,
            max_price_variation: old.max_price_variation,
            trading_reference_price: old.trading_reference_price,
            unit_of_measure_qty: old.unit_of_measure_qty,
            min_price_increment_amount: old.min_price_increment_amount,
            price_ratio: old.price_ratio,
            inst_attrib_value: old.inst_attrib_value,
            underlying_id: old.underlying_id,
            raw_instrument_id: old.raw_instrument_id,
            market_depth_implied: old.market_depth_implied,
            market_depth: old.market_depth,
            market_segment_id: old.market_segment_id,
            max_trade_vol: old.max_trade_vol,
            min_lot_size: old.min_lot_size,
            min_lot_size_block: old.min_lot_size_block,
            min_lot_size_round_lot: old.min_lot_size_round_lot,
            min_trade_vol: old.min_trade_vol,
            contract_multiplier: old.contract_multiplier,
            decay_quantity: old.decay_quantity,
            original_contract_size: old.original_contract_size,
            trading_reference_date: old.trading_reference_date,
            appl_id: old.appl_id,
            maturity_year: old.maturity_year,
            decay_start_date: old.decay_start_date,
            channel_id: old.channel_id,
            currency: old.currency,
            settl_currency: old.settl_currency,
            secsubtype: old.secsubtype,
            group: old.group,
            exchange: old.exchange,
            asset: old.asset,
            cfi: old.cfi,
            security_type: old.security_type,
            unit_of_measure: old.unit_of_measure,
            underlying: old.underlying,
            strike_price_currency: old.strike_price_currency,
            instrument_class: old.instrument_class,
            strike_price: old.strike_price,
            match_algorithm: old.match_algorithm,
            md_security_trading_status: old.md_security_trading_status,
            main_fraction: old.main_fraction,
            price_display_format: old.price_display_format,
            settl_price_type: old.settl_price_type,
            sub_fraction: old.sub_fraction,
            underlying_product: old.underlying_product,
            security_update_action: old.security_update_action as c_char,
            maturity_month: old.maturity_month,
            maturity_day: old.maturity_day,
            maturity_week: old.maturity_week,
            user_defined_instrument: old.user_defined_instrument,
            contract_multiplier_unit: old.contract_multiplier_unit,
            flow_schedule_type: old.flow_schedule_type,
            tick_rule: old.tick_rule,
            ..Default::default()
        };
        res.raw_symbol[..v1::SYMBOL_CSTR_LEN].copy_from_slice(old.raw_symbol.as_slice());
        res
    }
}

impl From<&v1::ErrorMsg> for ErrorMsg {
    fn from(old: &v1::ErrorMsg) -> Self {
        let mut new = Self {
            hd: RecordHeader::new::<Self>(
                rtype::ERROR,
                old.hd.publisher_id,
                old.hd.instrument_id,
                old.hd.ts_event,
            ),
            ..Default::default()
        };
        new.err[..old.err.len()].copy_from_slice(old.err.as_slice());
        new
    }
}

impl From<&v1::SymbolMappingMsg> for SymbolMappingMsg {
    fn from(old: &v1::SymbolMappingMsg) -> Self {
        let mut res = Self {
            hd: RecordHeader::new::<Self>(
                rtype::SYMBOL_MAPPING,
                old.hd.publisher_id,
                old.hd.instrument_id,
                old.hd.ts_event,
            ),
            start_ts: old.start_ts,
            end_ts: old.end_ts,
            ..Default::default()
        };
        res.stype_in_symbol[..v1::SYMBOL_CSTR_LEN].copy_from_slice(old.stype_in_symbol.as_slice());
        res.stype_out_symbol[..v1::SYMBOL_CSTR_LEN]
            .copy_from_slice(old.stype_out_symbol.as_slice());
        res
    }
}

impl From<&v1::SystemMsg> for SystemMsg {
    fn from(old: &v1::SystemMsg) -> Self {
        let mut new = Self {
            hd: RecordHeader::new::<Self>(
                rtype::SYSTEM,
                old.hd.publisher_id,
                old.hd.instrument_id,
                old.hd.ts_event,
            ),
            ..Default::default()
        };
        new.msg[..old.msg.len()].copy_from_slice(old.msg.as_slice());
        new
    }
}

impl SymbolMappingRec for SymbolMappingMsg {
    fn stype_in_symbol(&self) -> crate::Result<&str> {
        Self::stype_in_symbol(self)
    }

    fn stype_out_symbol(&self) -> crate::Result<&str> {
        Self::stype_out_symbol(self)
    }

    fn start_ts(&self) -> Option<time::OffsetDateTime> {
        Self::start_ts(self)
    }

    fn end_ts(&self) -> Option<time::OffsetDateTime> {
        Self::end_ts(self)
    }
}

impl InstrumentDefRec for InstrumentDefMsg {
    fn raw_symbol(&self) -> crate::Result<&str> {
        Self::raw_symbol(self)
    }

    fn asset(&self) -> crate::Result<&str> {
        Self::asset(self)
    }

    fn security_type(&self) -> crate::Result<&str> {
        Self::security_type(self)
    }

    fn security_update_action(&self) -> crate::Result<crate::SecurityUpdateAction> {
        Self::security_update_action(self)
    }

    fn channel_id(&self) -> u16 {
        self.channel_id
    }
}
