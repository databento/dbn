use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU64,
};

use crate::{Error, SType, Schema};

use super::{MappingInterval, Metadata, SymbolMapping};

pub struct MetadataMerger {
    version: u8,
    dataset: String,
    schema: Option<Schema>,
    start: u64,
    end: Option<NonZeroU64>,
    limit: Option<NonZeroU64>,
    stype_in: Option<SType>,
    stype_out: SType,
    ts_out: bool,
    symbol_cstr_len: usize,
    symbols: HashSet<String>,
    partial: HashSet<String>,
    not_found: HashSet<String>,
    mappings: HashMap<String, Vec<MappingInterval>>,
}

impl MetadataMerger {
    pub fn new(original: Metadata) -> Self {
        Self {
            version: original.version,
            dataset: original.dataset,
            schema: original.schema,
            start: original.start,
            end: original.end,
            limit: original.limit,
            stype_in: original.stype_in,
            stype_out: original.stype_out,
            ts_out: original.ts_out,
            symbol_cstr_len: original.symbol_cstr_len,
            symbols: original.symbols.into_iter().collect(),
            partial: original.partial.into_iter().collect(),
            not_found: original.not_found.into_iter().collect(),
            mappings: original
                .mappings
                .into_iter()
                .map(|mapping| (mapping.raw_symbol, mapping.intervals))
                .collect(),
        }
    }

    pub fn merge(&mut self, additional: Metadata) -> crate::Result<&mut Self> {
        let bad_arg = |field| {
            Err(crate::Error::BadArgument {
                param_name: "additional".to_owned(),
                desc: format!("{field} mismatch when attempting to merge Metadata objects"),
            })
        };
        if self.version != additional.version {
            return bad_arg("version");
        }
        if self.dataset != additional.dataset {
            return bad_arg("dataset");
        }
        if self.stype_in != additional.stype_in {
            return bad_arg("stype_in");
        }
        if self.stype_out != additional.stype_out {
            return bad_arg("stype_out");
        }
        if self.ts_out != additional.ts_out {
            return bad_arg("ts_out");
        }
        if self.symbol_cstr_len != additional.symbol_cstr_len {
            return bad_arg("symbol_cstr_len");
        }
        if self.schema != additional.schema {
            self.schema = None
        }
        self.start = self.start.min(additional.start);
        self.end = self.end.max(additional.end);
        // limit loses meaning after merge
        self.limit = None;
        self.symbols.extend(additional.symbols);
        self.partial.extend(additional.partial);
        self.not_found.extend(additional.not_found);
        for add_mapping in additional.mappings {
            let Some(intervals) = self.mappings.get_mut(&add_mapping.raw_symbol) else {
                self.mappings
                    .insert(add_mapping.raw_symbol, add_mapping.intervals);
                continue;
            };
            let orig_intervals = std::mem::take(intervals);
            *intervals = Self::merge_intervals(orig_intervals, add_mapping.intervals)?;
        }

        Ok(self)
    }

    fn merge_intervals(
        orig_intervals: Vec<MappingInterval>,
        add_intervals: Vec<MappingInterval>,
    ) -> crate::Result<Vec<MappingInterval>> {
        let mut merged_intervals = Vec::new();
        let mut orig_intervals = orig_intervals.into_iter().peekable();
        let mut add_intervals = add_intervals.into_iter().peekable();
        loop {
            match (orig_intervals.peek(), add_intervals.peek()) {
                (Some(int), Some(other_int)) => match int.ordering(other_int)? {
                    IntervalOrdering::Less => merged_intervals.push(orig_intervals.next().unwrap()),
                    IntervalOrdering::Greater => {
                        merged_intervals.push(add_intervals.next().unwrap())
                    }
                    IntervalOrdering::Overlap => merged_intervals.push(
                        orig_intervals
                            .next()
                            .unwrap()
                            .merge(add_intervals.next().unwrap()),
                    ),
                    IntervalOrdering::Equal => {
                        // Advance both
                        add_intervals.next().unwrap();
                        merged_intervals.push(orig_intervals.next().unwrap());
                    }
                },
                (Some(_), None) => {
                    merged_intervals.push(orig_intervals.next().unwrap());
                }
                (None, Some(_)) => {
                    merged_intervals.push(add_intervals.next().unwrap());
                }
                (None, None) => {
                    return Ok(merged_intervals);
                }
            }
        }
    }

    pub fn finalize(self) -> Metadata {
        Metadata {
            version: self.version,
            dataset: self.dataset,
            schema: self.schema,
            start: self.start,
            end: self.end,
            limit: self.limit,
            stype_in: self.stype_in,
            stype_out: self.stype_out,
            ts_out: self.ts_out,
            symbol_cstr_len: self.symbol_cstr_len,
            symbols: self.symbols.into_iter().collect(),
            partial: self.partial.into_iter().collect(),
            not_found: self.not_found.into_iter().collect(),
            mappings: self
                .mappings
                .into_iter()
                .map(|(raw_symbol, intervals)| SymbolMapping {
                    raw_symbol,
                    intervals,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntervalOrdering {
    Less,
    Overlap,
    Equal,
    Greater,
}

impl MappingInterval {
    fn ordering(&self, other: &Self) -> crate::Result<IntervalOrdering> {
        if self.start_date == other.start_date && self.end_date == other.end_date {
            return if self.symbol == other.symbol {
                Ok(IntervalOrdering::Equal)
            } else {
                Err(Error::BadArgument {
                    param_name: "mappings".to_owned(),
                    desc: format!(
                        "conflicting intervals mapping to {} and {}",
                        self.symbol, other.symbol
                    ),
                })
            };
        }
        if self.end_date <= other.start_date {
            Ok(IntervalOrdering::Less)
        } else if self.start_date >= other.end_date {
            Ok(IntervalOrdering::Greater)
        } else if self.symbol == other.symbol {
            Ok(IntervalOrdering::Overlap)
        } else {
            Err(Error::BadArgument {
                param_name: "mappings".to_owned(),
                desc: format!("invalid mapping interval {self:?} {other:?}"),
            })
        }
    }

    fn merge(self, other: Self) -> Self {
        Self {
            start_date: self.start_date.min(other.start_date),
            end_date: self.end_date.max(other.end_date),
            symbol: self.symbol,
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use time::{
        macros::{date, datetime},
        Date,
    };

    use crate::symbol_map::tests::metadata_w_mappings;

    use super::*;

    #[test]
    fn test_merge() {
        let target = metadata_w_mappings();
        let other = Metadata::builder()
            .dataset(target.dataset.clone())
            .schema(target.schema)
            .stype_in(target.stype_in)
            .stype_out(target.stype_out)
            .start(datetime!(2023 - 06 -15 00:00 UTC).unix_timestamp_nanos() as u64)
            .end(NonZeroU64::new(
                datetime!(2023-07-15 00:00 UTC).unix_timestamp_nanos() as u64,
            ))
            .mappings(vec![
                // One symbol in `target`
                SymbolMapping {
                    raw_symbol: "MSFT".to_owned(),
                    intervals: vec![
                        MappingInterval {
                            start_date: date!(2023 - 06 - 15),
                            end_date: date!(2023 - 06 - 22),
                            symbol: "6000".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 06 - 22),
                            end_date: date!(2023 - 06 - 29),
                            symbol: "6001".to_owned(),
                        },
                        // should be merged
                        MappingInterval {
                            start_date: date!(2023 - 06 - 29),
                            end_date: date!(2023 - 07 - 03),
                            symbol: "6854".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 03),
                            end_date: date!(2023 - 07 - 05),
                            symbol: "6849".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 05),
                            end_date: date!(2023 - 07 - 06),
                            symbol: "6846".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 06),
                            end_date: date!(2023 - 07 - 07),
                            symbol: "6843".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 07),
                            end_date: date!(2023 - 07 - 10),
                            symbol: "6840".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 10),
                            end_date: date!(2023 - 07 - 11),
                            symbol: "6833".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 11),
                            end_date: date!(2023 - 07 - 12),
                            symbol: "6830".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 12),
                            end_date: date!(2023 - 07 - 13),
                            symbol: "6826".to_owned(),
                        },
                        // should be subsumed by the more complete mapping in target
                        MappingInterval {
                            start_date: date!(2023 - 07 - 13),
                            end_date: date!(2023 - 07 - 15),
                            symbol: "6827".to_owned(),
                        },
                    ],
                },
                // One symbol not in `target`
                SymbolMapping {
                    raw_symbol: "META".to_owned(),
                    intervals: vec![
                        MappingInterval {
                            start_date: date!(2023 - 06 - 15),
                            end_date: date!(2023 - 07 - 01),
                            symbol: "400".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 01),
                            end_date: date!(2023 - 07 - 08),
                            symbol: "401".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 08),
                            end_date: date!(2023 - 07 - 15),
                            symbol: "399".to_owned(),
                        },
                    ],
                },
            ])
            .build();
        let merged = target.clone().merge(vec![other.clone()]).unwrap();
        assert_eq!(merged.start, other.start);
        assert_eq!(merged.end, target.end);
        let msft_mapping = merged
            .mappings
            .iter()
            .find(|mapping| mapping.raw_symbol == "MSFT")
            .unwrap();
        assert_eq!(
            msft_mapping.intervals.first().unwrap(),
            &other.mappings[0].intervals[0]
        );
        assert_eq!(
            msft_mapping.intervals.last().unwrap(),
            target.mappings[2].intervals.last().unwrap()
        );
        assert!(msft_mapping
            .intervals
            .iter()
            .any(|interval| interval.symbol == "6854"
                && interval.start_date == date!(2023 - 06 - 29)
                && interval.end_date == date!(2023 - 07 - 03)));
        assert!(msft_mapping
            .intervals
            .iter()
            .any(|interval| interval.symbol == "6827"
                && interval.start_date == date!(2023 - 07 - 13)
                && interval.end_date == date!(2023 - 07 - 17)));
        let meta_mapping = merged
            .mappings
            .iter()
            .find(|mapping| mapping.raw_symbol == "META")
            .unwrap();
        assert_eq!(meta_mapping, &other.mappings[1]);
        for symbol in ["AAPL", "NVDA", "PLTR"] {
            let is_symbol = |mapping: &&SymbolMapping| mapping.raw_symbol == symbol;
            let merge_mapping = merged.mappings.iter().find(is_symbol).unwrap();
            let orig_mapping = target.mappings.iter().find(is_symbol).unwrap();
            assert_eq!(merge_mapping, orig_mapping);
        }
        merged.symbol_map().unwrap();
    }

    #[fixture]
    fn mapping_interval() -> MappingInterval {
        MappingInterval {
            start_date: date!(2024 - 12 - 16),
            end_date: date!(2024 - 12 - 21),
            symbol: "AAPL".to_owned(),
        }
    }

    #[rstest]
    #[case::less(date!(2024-12-21), date!(2025-01-01), IntervalOrdering::Less)]
    #[case::eq(date!(2024-12-16), date!(2024-12-21), IntervalOrdering::Equal)]
    #[case::greater(date!(2024-12-01), date!(2024-12-16), IntervalOrdering::Greater)]
    #[case::overlap_wider(date!(2024-12-17), date!(2024-12-18), IntervalOrdering::Overlap)]
    #[case::overlap_narrower(date!(2024-01-01), date!(2024-12-31), IntervalOrdering::Overlap)]
    #[case::overlap_earlier(date!(2024-12-18), date!(2024-12-31), IntervalOrdering::Overlap)]
    #[case::overlap_later(date!(2024-11-16), date!(2024-12-22), IntervalOrdering::Overlap)]
    fn mapping_interval_ordering(
        mapping_interval: MappingInterval,
        #[case] start_date: Date,
        #[case] end_date: Date,
        #[case] exp: IntervalOrdering,
    ) {
        let other = MappingInterval {
            start_date,
            end_date,
            symbol: mapping_interval.symbol.clone(),
        };
        assert_eq!(mapping_interval.ordering(&other).unwrap(), exp);
    }

    #[rstest]
    fn mapping_interval_ordering_diff_symbol(mapping_interval: MappingInterval) {
        let other = MappingInterval {
            start_date: date!(2024 - 12 - 16),
            end_date: date!(2024 - 12 - 21),
            symbol: "AMZN".to_owned(),
        };
        let res = mapping_interval.ordering(&other);
        dbg!(&res);
        assert!(
            matches!(res, Err(Error::BadArgument { param_name, desc }) if param_name == "mappings" && desc.contains("AAPL and AMZN"))
        );
    }
}
