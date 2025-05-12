use std::os::raw::c_char;

use crate::{rtype, v1, v2, RecordHeader};

use super::{InstrumentDefMsg, StatMsg, UNDEF_STAT_QUANTITY};

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
            unit_of_measure_qty: old.unit_of_measure_qty,
            min_price_increment_amount: old.min_price_increment_amount,
            price_ratio: old.price_ratio,
            inst_attrib_value: old.inst_attrib_value,
            underlying_id: old.underlying_id,
            raw_instrument_id: u64::from(old.raw_instrument_id),
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
            appl_id: old.appl_id,
            maturity_year: old.maturity_year,
            decay_start_date: old.decay_start_date,
            channel_id: old.channel_id,
            currency: old.currency,
            settl_currency: old.settl_currency,
            secsubtype: old.secsubtype,
            group: old.group,
            exchange: old.exchange,
            cfi: old.cfi,
            security_type: old.security_type,
            unit_of_measure: old.unit_of_measure,
            underlying: old.underlying,
            strike_price_currency: old.strike_price_currency,
            instrument_class: old.instrument_class,
            strike_price: old.strike_price,
            match_algorithm: old.match_algorithm,
            main_fraction: old.main_fraction,
            price_display_format: old.price_display_format,
            sub_fraction: old.sub_fraction,
            underlying_product: old.underlying_product,
            security_update_action: old.security_update_action as c_char,
            maturity_month: old.maturity_month,
            maturity_day: old.maturity_day,
            maturity_week: old.maturity_week,
            user_defined_instrument: old.user_defined_instrument as c_char,
            contract_multiplier_unit: old.contract_multiplier_unit,
            flow_schedule_type: old.flow_schedule_type,
            tick_rule: old.tick_rule,
            ..Default::default()
        };
        res.asset[..v1::ASSET_CSTR_LEN].copy_from_slice(old.asset.as_slice());
        res.raw_symbol[..v1::SYMBOL_CSTR_LEN].copy_from_slice(old.raw_symbol.as_slice());
        res
    }
}

impl From<&v2::InstrumentDefMsg> for InstrumentDefMsg {
    fn from(old: &v2::InstrumentDefMsg) -> Self {
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
            unit_of_measure_qty: old.unit_of_measure_qty,
            min_price_increment_amount: old.min_price_increment_amount,
            price_ratio: old.price_ratio,
            inst_attrib_value: old.inst_attrib_value,
            underlying_id: old.underlying_id,
            raw_instrument_id: u64::from(old.raw_instrument_id),
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
            appl_id: old.appl_id,
            maturity_year: old.maturity_year,
            decay_start_date: old.decay_start_date,
            channel_id: old.channel_id,
            currency: old.currency,
            settl_currency: old.settl_currency,
            secsubtype: old.secsubtype,
            group: old.group,
            exchange: old.exchange,
            cfi: old.cfi,
            security_type: old.security_type,
            unit_of_measure: old.unit_of_measure,
            underlying: old.underlying,
            strike_price_currency: old.strike_price_currency,
            instrument_class: old.instrument_class,
            strike_price: old.strike_price,
            match_algorithm: old.match_algorithm,
            main_fraction: old.main_fraction,
            price_display_format: old.price_display_format,
            sub_fraction: old.sub_fraction,
            underlying_product: old.underlying_product,
            security_update_action: old.security_update_action as c_char,
            maturity_month: old.maturity_month,
            maturity_day: old.maturity_day,
            maturity_week: old.maturity_week,
            user_defined_instrument: old.user_defined_instrument as c_char,
            contract_multiplier_unit: old.contract_multiplier_unit,
            flow_schedule_type: old.flow_schedule_type,
            tick_rule: old.tick_rule,
            raw_symbol: old.raw_symbol,
            ..Default::default()
        };
        res.asset[..v2::ASSET_CSTR_LEN].copy_from_slice(old.asset.as_slice());
        res
    }
}

impl From<&v1::StatMsg> for StatMsg {
    fn from(old: &v1::StatMsg) -> Self {
        Self {
            // recalculate length
            hd: RecordHeader::new::<Self>(
                rtype::STATISTICS,
                old.hd.publisher_id,
                old.hd.instrument_id,
                old.hd.ts_event,
            ),
            ts_recv: old.ts_recv,
            ts_ref: old.ts_ref,
            price: old.price,
            quantity: if old.quantity == v1::UNDEF_STAT_QUANTITY {
                UNDEF_STAT_QUANTITY
            } else {
                old.quantity as i64
            },
            sequence: old.sequence,
            ts_in_delta: old.ts_in_delta,
            stat_type: old.stat_type,
            channel_id: old.channel_id,
            update_action: old.update_action,
            stat_flags: old.stat_flags,
            _reserved: Default::default(),
        }
    }
}
