//! **The prices pane's fixture** (bl-9111) — a channel-wide pane, kept in its
//! own file so the next window-level pane does not land in the same lines.

use crate::reply::prices::{Prices, Rate};
use crate::reply::spend::Cost;
use crate::ui::Model;

use super::super::{own, seated};

/// One row, at the rates given, under a wildcard model.
pub(crate) fn rate(provider: &str, input: u64, output: u64) -> Rate {
    Rate {
        provider: provider.to_owned(),
        model: "*".to_owned(),
        input: input.into(),
        output: output.into(),
        cache_read: 0_u64.into(),
        cache_write: 0_u64.into(),
    }
}

/// **A table with every sentence in it**: a priced row and a subscription, a
/// bound, a spend that is a floor, and a receipt that woke two.
pub(crate) fn table() -> Prices {
    Prices {
        rows: vec![rate("housevendor", 3, 15), rate("subscription", 0, 0)],
        ceiling: Some(25_u64.into()),
        spent: Some(Cost {
            usd: "$4.25".to_owned(),
            unpriced_tokens: 12,
        }),
        released: Some(2),
    }
}

/// **The seated model with the prices pane open and answered.**
pub(crate) fn priced() -> Model {
    Model {
        lookup: Some(crate::ui::Lookup::Pricing),
        pricing: crate::ui::Pricing {
            tables: vec![crate::ui::PriceTable {
                channel: own().channel,
                prices: table(),
            }],
            ..crate::ui::Pricing::default()
        },
        ..seated()
    }
}
