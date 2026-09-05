//! The one field: which listing is up, and that putting one down never takes
//! another with it.

use super::Listing;
use crate::ui::Model;

/// **A pane is up or it is not**, and the field is the whole of it — which is
/// what makes *two listings on one glass* unrepresentable rather than merely
/// unreachable.
#[test]
fn one_field_holds_which_listing_is_standing() {
    let mut model = Model::default();
    assert!(!model.showing(Listing::Queue));
    model.stand(Listing::Queue);
    assert!(model.showing(Listing::Queue));
    assert!(!model.showing(Listing::Records), "and only that one");
    model.stand(Listing::Records);
    assert!(model.showing(Listing::Records), "standing one replaces it");
}

/// **A close control names its own pane**, so a stale one cannot take down the
/// pane that replaced it.
#[test]
fn putting_one_down_leaves_another_standing() {
    let mut model = Model::default();
    model.stand(Listing::Clients);
    model.put_down(Listing::Records);
    assert!(
        model.showing(Listing::Clients),
        "the wrong word does nothing"
    );
    model.put_down(Listing::Clients);
    assert_eq!(model.listing, None);
}
