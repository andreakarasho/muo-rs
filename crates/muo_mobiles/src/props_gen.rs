//! The `[CommandProperty]` tables of classes whose members are fields of a data struct in `ext`.
#![allow(
    unused_imports,
    unused_variables,
    dead_code,
    redundant_semicolons,
    clippy::all
)]
use muo_content::aos::AosElementAttribute;
use muo_content::props::{self, Kind, PropDef, Value};
use muo_content::utilities::types::{Numeric, PropType};
use muo_primitives::{DateTime, Point2D, Point3D, Rectangle2D, TimeSpan};
use muo_server::ids::EntityId;
use muo_server::mobile::AccessLevel;
use muo_server::world::World;

const PLAYER: AccessLevel = AccessLevel::Player;
const COUNSELOR: AccessLevel = AccessLevel::Counselor;
const GM: AccessLevel = AccessLevel::GameMaster;
const SEER: AccessLevel = AccessLevel::Seer;
const ADMIN: AccessLevel = AccessLevel::Administrator;
const DEV: AccessLevel = AccessLevel::Developer;
const OWNER: AccessLevel = AccessLevel::Owner;
const BOOL: Kind = Kind::Plain(PropType::Bool);
const STR: Kind = Kind::Plain(PropType::Str);
const ENTITY: Kind = Kind::Plain(PropType::Entity);
const NOT_HERE: &str = "That object has no such state.";
const FACTION: Kind = Kind::Nullable(&Kind::CustomEnum(
    &[
        "Minax",
        "Council of Mages",
        "True Britannians",
        "Shadowlords",
    ],
    &[
        ("Council of Mages", 0),
        ("Minax", 1),
        ("Shadowlords", 2),
        ("True Britannians", 3),
    ],
));
const TOWN: Kind = Kind::Nullable(&Kind::CustomEnum(
    &[
        "Britain",
        "Magincia",
        "Minoc",
        "Moonglow",
        "Skara Brae",
        "Trinsic",
        "Vesper",
        "Yew",
    ],
    &[
        ("Britain", 0),
        ("Magincia", 1),
        ("Minoc", 2),
        ("Moonglow", 3),
        ("Skara Brae", 4),
        ("Trinsic", 5),
        ("Vesper", 6),
        ("Yew", 7),
    ],
));

const E_CRAFTRESOURCE: &[(&str, i64)] = &[
    ("None", 0),
    ("Iron", 1),
    ("DullCopper", 2),
    ("ShadowIron", 3),
    ("Copper", 4),
    ("Bronze", 5),
    ("Gold", 6),
    ("Agapite", 7),
    ("Verite", 8),
    ("Valorite", 9),
    ("RegularLeather", 101),
    ("SpinedLeather", 102),
    ("HornedLeather", 103),
    ("BarbedLeather", 104),
    ("RedScales", 201),
    ("YellowScales", 202),
    ("BlackScales", 203),
    ("GreenScales", 204),
    ("WhiteScales", 205),
    ("BlueScales", 206),
    ("RegularWood", 301),
    ("OakWood", 302),
    ("AshWood", 303),
    ("YewWood", 304),
    ("Heartwood", 305),
    ("Bloodwood", 306),
    ("Frostwood", 307),
];
static SERVER_ITEMS_VENDORRENTALCONTRACT: &[PropDef] = &[
    PropDef {
        name: "Price",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| {
                    o.ext
                        .get::<crate::npcs::vendors::rental_contract::ContractData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.price;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<crate::npcs::vendors::rental_contract::ContractData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.price == n {
                return Ok(());
            }
            d.price = n;
            Ok(())
        }),
    },
    PropDef {
        name: "LandlordRenew",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| {
                    o.ext
                        .get::<crate::npcs::vendors::rental_contract::ContractData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.landlord_renew;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            let Some(d) = w.item_mut(i).and_then(|o| {
                o.ext
                    .get_mut::<crate::npcs::vendors::rental_contract::ContractData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.landlord_renew == n {
                return Ok(());
            }
            d.landlord_renew = n;
            Ok(())
        }),
    },
];

static SERVER_MOBILES_ETHEREALMOUNT: &[PropDef] = &[
    PropDef {
        name: "IsDonationItem",
        kind: BOOL,
        read: GM,
        write: ADMIN,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::classes::ethereal::EtherealData>())
                .map_or(Value::Null, |d| {
                    let x = &d.is_donation_item;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::classes::ethereal::EtherealData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.is_donation_item == n {
                return Ok(());
            }
            d.is_donation_item = n;
            Ok(())
        }),
    },
    PropDef {
        name: "IsRewardItem",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::classes::ethereal::EtherealData>())
                .map_or(Value::Null, |d| {
                    let x = &d.is_reward_item;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::classes::ethereal::set_is_reward_item(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "MountedID",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::classes::ethereal::EtherealData>())
                .map_or(Value::Null, |d| {
                    let x = &d.mounted_id;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::classes::ethereal::set_mounted_id(w, i, n);
            Ok(())
        }),
    },
    PropDef {
        name: "RegularID",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::classes::ethereal::EtherealData>())
                .map_or(Value::Null, |d| {
                    let x = &d.regular_id;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::classes::ethereal::set_regular_id(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_MOBILES_BASEHEALER: &[PropDef] = &[PropDef {
    name: "Price",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Mobile(m) => w
            .mobile(m)
            .and_then(|o| o.ext.get::<crate::npcs::healers::HealerData>())
            .map_or(Value::Null, |d| {
                let x = &d.price;
                Value::Int(i64::from(*x))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Mobile(m) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.i64() as i32;
        crate::npcs::healers::set_price(w, m, n);
        Ok(())
    }),
}];

static SERVER_MOBILES_BASEHIRE: &[PropDef] = &[
    PropDef {
        name: "NextPay",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::npcs::hires::HireData>())
                .map_or(Value::Null, |d| {
                    let x = &d.next_pay;
                    Value::DateTime(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::DateTime(t) => t,
                _ => DateTime::MIN,
            };
            let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::npcs::hires::HireData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.next_pay == n {
                return Ok(());
            }
            d.next_pay = n;
            Ok(())
        }),
    },
    PropDef {
        name: "HoldGold",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::npcs::hires::HireData>())
                .map_or(Value::Null, |d| {
                    let x = &d.hold_gold;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::npcs::hires::HireData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.hold_gold == n {
                return Ok(());
            }
            d.hold_gold = n;
            Ok(())
        }),
    },
    PropDef {
        name: "IsHired",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::npcs::hires::HireData>())
                .map_or(Value::Null, |d| {
                    let x = &d.is_hired;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::npcs::hires::set_is_hired(w, m, n);
            Ok(())
        }),
    },
];

static SERVER_MOBILES_PLAYERVENDOR: &[PropDef] = &[
    PropDef {
        name: "ShopName",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| {
                    o.ext
                        .get::<crate::npcs::vendors::player_vendor::PlayerVendorData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.shop_name;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            crate::npcs::vendors::player_vendor::set_shop_name(w, m, n);
            Ok(())
        }),
    },
    PropDef {
        name: "NextPayTime",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| {
                    o.ext
                        .get::<crate::npcs::vendors::player_vendor::PlayerVendorData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.next_pay_time;
                    Value::DateTime(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::DateTime(t) => t,
                _ => DateTime::MIN,
            };
            let Some(d) = w.mobile_mut(m).and_then(|o| {
                o.ext
                    .get_mut::<crate::npcs::vendors::player_vendor::PlayerVendorData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.next_pay_time == n {
                return Ok(());
            }
            d.next_pay_time = n;
            Ok(())
        }),
    },
    PropDef {
        name: "Owner",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| {
                    o.ext
                        .get::<crate::npcs::vendors::player_vendor::PlayerVendorData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.owner;
                    Value::Entity(x.map(EntityId::Mobile))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            let Some(d) = w.mobile_mut(m).and_then(|o| {
                o.ext
                    .get_mut::<crate::npcs::vendors::player_vendor::PlayerVendorData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.owner == n {
                return Ok(());
            }
            d.owner = n;
            Ok(())
        }),
    },
    PropDef {
        name: "BankAccount",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| {
                    o.ext
                        .get::<crate::npcs::vendors::player_vendor::PlayerVendorData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.bank_account;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::npcs::vendors::player_vendor::set_bank_account(w, m, n);
            Ok(())
        }),
    },
    PropDef {
        name: "HoldGold",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| {
                    o.ext
                        .get::<crate::npcs::vendors::player_vendor::PlayerVendorData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.hold_gold;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            crate::npcs::vendors::player_vendor::set_hold_gold(w, m, n);
            Ok(())
        }),
    },
    PropDef {
        name: "SellItems",
        kind: Kind::Text,
        read: GM,
        write: GM,
        get: |w, e| {
            match e { EntityId::Mobile(m) => w.mobile(m).and_then(|o| o.ext.get::<crate::npcs::vendors::player_vendor::PlayerVendorData>()).map_or(Value::Null, |d| { let x = &d.sell_items; Value::Text("System.Collections.Generic.Dictionary`2[Server.Item,Server.Mobiles.VendorItem]".to_string()) }), _ => Value::Null }
        },
        set: None,
    },
];

static SERVER_MOBILES_PLAYERVENDORPLACEHOLDER: &[PropDef] = &[PropDef {
    name: "Vendor",
    kind: Kind::Plain(PropType::EntityOf("PlayerVendor")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| {
                o.ext
                    .get::<crate::npcs::vendors::player_vendor::PlaceholderData>()
            })
            .map_or(Value::Null, |d| {
                let x = &d.vendor;
                Value::Entity(x.map(EntityId::Mobile))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.mobile();
        let Some(d) = w.item_mut(i).and_then(|o| {
            o.ext
                .get_mut::<crate::npcs::vendors::player_vendor::PlaceholderData>()
        }) else {
            return Err(NOT_HERE.to_string());
        };
        if d.vendor == n {
            return Ok(());
        }
        d.vendor = n;
        Ok(())
    }),
}];

static SERVER_MOBILES_RENTEDVENDOR: &[PropDef] = &[
    PropDef {
        name: "RentalPrice",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::npcs::vendors::rented::RentedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.rental_price;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::npcs::vendors::rented::RentedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.rental_price == n {
                return Ok(());
            }
            d.rental_price = n;
            Ok(())
        }),
    },
    PropDef {
        name: "LandlordRenew",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::npcs::vendors::rented::RentedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.landlord_renew;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::npcs::vendors::rented::RentedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.landlord_renew == n {
                return Ok(());
            }
            d.landlord_renew = n;
            Ok(())
        }),
    },
    PropDef {
        name: "RenterRenew",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::npcs::vendors::rented::RentedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.renter_renew;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::npcs::vendors::rented::RentedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.renter_renew == n {
                return Ok(());
            }
            d.renter_renew = n;
            Ok(())
        }),
    },
    PropDef {
        name: "RenewalPrice",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::npcs::vendors::rented::RentedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.renewal_price;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::npcs::vendors::rented::RentedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.renewal_price == n {
                return Ok(());
            }
            d.renewal_price = n;
            Ok(())
        }),
    },
    PropDef {
        name: "RentalGold",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::npcs::vendors::rented::RentedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.rental_gold;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::npcs::vendors::rented::RentedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.rental_gold == n {
                return Ok(());
            }
            d.rental_gold = n;
            Ok(())
        }),
    },
    PropDef {
        name: "RentalExpireTime",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::npcs::vendors::rented::RentedData>())
                .map_or(Value::Null, |d| {
                    let x = &d.rental_expire_time;
                    Value::DateTime(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::DateTime(t) => t,
                _ => DateTime::MIN,
            };
            let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::npcs::vendors::rented::RentedData>())
            else {
                return Err(NOT_HERE.to_string());
            };
            if d.rental_expire_time == n {
                return Ok(());
            }
            d.rental_expire_time = n;
            Ok(())
        }),
    },
];

static SERVER_MOBILES_PLAYERBARKEEPER: &[PropDef] = &[
    PropDef {
        name: "Owner",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| {
                    o.ext
                        .get::<crate::npcs::vendors::barkeeper::BarkeeperData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.owner;
                    Value::Entity(x.map(EntityId::Mobile))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            let Some(d) = w.mobile_mut(m).and_then(|o| {
                o.ext
                    .get_mut::<crate::npcs::vendors::barkeeper::BarkeeperData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.owner == n {
                return Ok(());
            }
            d.owner = n;
            Ok(())
        }),
    },
    PropDef {
        name: "TipMessage",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| {
                    o.ext
                        .get::<crate::npcs::vendors::barkeeper::BarkeeperData>()
                })
                .map_or(Value::Null, |d| {
                    let x = &d.tip_message;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            let Some(d) = w.mobile_mut(m).and_then(|o| {
                o.ext
                    .get_mut::<crate::npcs::vendors::barkeeper::BarkeeperData>()
            }) else {
                return Err(NOT_HERE.to_string());
            };
            if d.tip_message == n {
                return Ok(());
            }
            d.tip_message = n;
            Ok(())
        }),
    },
];

static SERVER_MOBILES_COW: &[PropDef] = &[
    PropDef {
        name: "MilkedOn",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::hand::animals::CowData>())
                .map_or(Value::Null, |d| {
                    let x = &d.milked_on;
                    Value::DateTime(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::DateTime(t) => t,
                _ => DateTime::MIN,
            };
            if let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::hand::animals::CowData>())
            {
                d.milked_on = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Milk",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::hand::animals::CowData>())
                .map_or(Value::Null, |d| {
                    let x = &d.milk;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::hand::animals::CowData>())
            {
                d.milk = n;
            };
            Ok(())
        }),
    },
];

static SERVER_MOBILES_SHEEP: &[PropDef] = &[PropDef {
    name: "NextWoolTime",
    kind: Kind::DateTime,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Mobile(m) => w
            .mobile(m)
            .and_then(|o| o.ext.get::<crate::hand::animals::SheepData>())
            .map_or(Value::Null, |d| {
                let x = &d.next_wool;
                Value::DateTime(*x)
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Mobile(m) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = match v {
            Value::DateTime(t) => t,
            _ => DateTime::MIN,
        };
        {
            let changed = w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::hand::animals::SheepData>())
                .is_some_and(|d| d.next_wool != n);
            if changed {
                if let Some(d) = w
                    .mobile_mut(m)
                    .and_then(|o| o.ext.get_mut::<crate::hand::animals::SheepData>())
                {
                    d.next_wool = n;
                }
                crate::hand::animals::sheep_body(w, m);
            }
        };
        Ok(())
    }),
}];

static SERVER_MOBILES_WATERELEMENTAL: &[PropDef] = &[PropDef {
    name: "HasDecanter",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Mobile(m) => Value::Bool(
            w.mobile(m)
                .and_then(|o| o.ext.get::<crate::hand::animals::HasDecanter>())
                .is_some_and(|d| d.0),
        ),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Mobile(m) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.bool();
        if let Some(o) = w.mobile_mut(m) {
            o.ext.insert(crate::hand::animals::HasDecanter(n));
        };
        Ok(())
    }),
}];

static SERVER_MOBILES_SWAMPDRAGON: &[PropDef] = &[
    PropDef {
        name: "BardingExceptional",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::classes::swamp_dragon::SwampDragonData>())
                .map_or(Value::Null, |d| {
                    let x = &d.barding_exceptional;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            if let Some(d) = w.mobile_mut(m).and_then(|o| {
                o.ext
                    .get_mut::<crate::classes::swamp_dragon::SwampDragonData>()
            }) {
                d.barding_exceptional = n;
            }
            muo_server::mobile::part2::properties::invalidate_properties(w, m);
            Ok(())
        }),
    },
    PropDef {
        name: "BardingCraftedBy",
        kind: STR,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::classes::swamp_dragon::SwampDragonData>())
                .map_or(Value::Null, |d| {
                    let x = &d.barding_crafted_by;
                    x.clone().map_or(Value::Null, Value::Str)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.string();
            if let Some(d) = w.mobile_mut(m).and_then(|o| {
                o.ext
                    .get_mut::<crate::classes::swamp_dragon::SwampDragonData>()
            }) {
                d.barding_crafted_by = n;
            }
            muo_server::mobile::part2::properties::invalidate_properties(w, m);
            Ok(())
        }),
    },
    PropDef {
        name: "BardingHP",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::classes::swamp_dragon::SwampDragonData>())
                .map_or(Value::Null, |d| {
                    let x = &d.barding_hp;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w.mobile_mut(m).and_then(|o| {
                o.ext
                    .get_mut::<crate::classes::swamp_dragon::SwampDragonData>()
            }) {
                d.barding_hp = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "HasBarding",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::classes::swamp_dragon::SwampDragonData>())
                .map_or(Value::Null, |d| {
                    let x = &d.has_barding;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            crate::classes::swamp_dragon::set_has_barding(w, m, n);
            Ok(())
        }),
    },
    PropDef {
        name: "BardingResource",
        kind: Kind::Plain(PropType::Enum(E_CRAFTRESOURCE)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::classes::swamp_dragon::SwampDragonData>())
                .map_or(Value::Null, |d| Value::Enum(d.barding_resource as i64)),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            match muo_content::craft::CraftResource::from_i32(n) {
                Some(r) => crate::classes::swamp_dragon::set_barding_resource(w, m, r),
                None => return Err("That is not a member of the enum.".to_string()),
            };
            Ok(())
        }),
    },
    PropDef {
        name: "BardingMaxHP",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => Value::Int(i64::from(
                crate::classes::swamp_dragon::barding_max_hp(w, m),
            )),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_MOBILES_PLAGUEBEAST: &[PropDef] = &[
    PropDef {
        name: "HasMetalChest",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::hand::batch5d::PlagueBeastData>())
                .map_or(Value::Null, |d| {
                    let x = &d.has_metal_chest;
                    Value::Bool(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.bool();
            if let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::hand::batch5d::PlagueBeastData>())
            {
                d.has_metal_chest = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "TotalDevoured",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::hand::batch5d::PlagueBeastData>())
                .map_or(Value::Null, |d| {
                    let x = &d.total_devoured;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::hand::batch5d::PlagueBeastData>())
            {
                d.total_devoured = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "DevourGoal",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => Value::Int(i64::from(
                crate::hand::batch5d::plague_beast_devour_goal(w, m),
            )),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::hand::batch5d::PlagueBeastData>())
            {
                d.devour_goal = n;
            };
            Ok(())
        }),
    },
];

static SERVER_MOBILES_PLAGUEBEASTLORD: &[PropDef] = &[
    PropDef {
        name: "OpenedBy",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => Value::Entity(
                crate::hand::batch5g::state(w, m)
                    .opened_by
                    .map(EntityId::Mobile),
            ),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            {
                let mut s = crate::hand::batch5g::state(w, m);
                s.opened_by = n;
                crate::hand::batch5g::set_state(w, m, s);
            };
            Ok(())
        }),
    },
    PropDef {
        name: "IsBleeding",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => Value::Bool(muo_items::special::mutation::is_bleeding(w, m)),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_MOBILES_PLAGUESPAWN: &[PropDef] = &[
    PropDef {
        name: "Owner",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::classes::misc4::PlagueSpawnState>())
                .map_or(Value::Null, |d| {
                    let x = &d.owner;
                    Value::Entity(x.map(EntityId::Mobile))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            if let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::classes::misc4::PlagueSpawnState>())
            {
                d.owner = n;
            };
            Ok(())
        }),
    },
    PropDef {
        name: "ExpireTime",
        kind: Kind::DateTime,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => w
                .mobile(m)
                .and_then(|o| o.ext.get::<crate::classes::misc4::PlagueSpawnState>())
                .map_or(Value::Null, |d| {
                    let x = &d.expire;
                    Value::DateTime(*x)
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Mobile(m) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = match v {
                Value::DateTime(t) => t,
                _ => DateTime::MIN,
            };
            if let Some(d) = w
                .mobile_mut(m)
                .and_then(|o| o.ext.get_mut::<crate::classes::misc4::PlagueSpawnState>())
            {
                d.expire = n;
            };
            Ok(())
        }),
    },
];

static SERVER_MOBILES_STAINEDOOZE: &[PropDef] = &[PropDef {
    name: "Corrosive",
    kind: BOOL,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Item(i) => w
            .item(i)
            .and_then(|o| o.ext.get::<crate::classes::ilhenir::OozeData>())
            .map_or(Value::Null, |d| {
                let x = &d.corrosive;
                Value::Bool(*x)
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Item(i) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.bool();
        if let Some(d) = w
            .item_mut(i)
            .and_then(|o| o.ext.get_mut::<crate::classes::ilhenir::OozeData>())
        {
            d.corrosive = n;
        };
        Ok(())
    }),
}];

static SERVER_MOBILES_CHANGELING: &[PropDef] = &[PropDef {
    name: "MorphedInto",
    kind: Kind::Plain(PropType::EntityOf("Mobile")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Mobile(m) => w
            .mobile(m)
            .and_then(|o| o.ext.get::<crate::hand::batch5f::ChangelingData>())
            .map_or(Value::Null, |d| {
                let x = &d.morphed_into;
                Value::Entity(x.map(EntityId::Mobile))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Mobile(m) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.mobile();
        crate::hand::batch5f::changeling_set_morphed_into(w, m, n);
        Ok(())
    }),
}];

static SERVER_MOBILES_HARROWERTENTACLES: &[PropDef] = &[PropDef {
    name: "Harrower",
    kind: Kind::Plain(PropType::EntityOf("Harrower")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Mobile(m) => w
            .mobile(m)
            .and_then(|o| o.ext.get::<crate::npcs::special::harrower::TentaclesData>())
            .map_or(Value::Null, |d| {
                let x = &d.harrower;
                Value::Entity(x.map(EntityId::Mobile))
            }),
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Mobile(m) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.mobile();
        if let Some(d) = w.mobile_mut(m).and_then(|o| {
            o.ext
                .get_mut::<crate::npcs::special::harrower::TentaclesData>()
        }) {
            d.harrower = n;
        };
        Ok(())
    }),
}];

static SERVER_MOBILES_ARCHERGUARD: &[PropDef] = &[PropDef {
    name: "Focus",
    kind: Kind::Plain(PropType::EntityOf("Mobile")),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Mobile(m) => {
            Value::Entity(crate::npcs::guards::focus(w, m).map(EntityId::Mobile))
        }
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Mobile(m) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.mobile();
        crate::npcs::guards::set_focus(w, m, n);
        Ok(())
    }),
}];

static SERVER_MOBILES_BASEHIRE_19: &[PropDef] = &[PropDef {
    name: "Pay",
    kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Mobile(m) => Value::Int(i64::from(crate::npcs::hires::per_day_cost(w, m))),
        _ => Value::Null,
    },
    set: None,
}];

static SERVER_MOBILES_RENTEDVENDOR_20: &[PropDef] = &[
    PropDef {
        name: "Landlord",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => {
                Value::Entity(crate::npcs::vendors::rented::landlord(w, m).map(EntityId::Mobile))
            }
            _ => Value::Null,
        },
        set: None,
    },
    PropDef {
        name: "Renew",
        kind: BOOL,
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Mobile(m) => Value::Bool(crate::npcs::vendors::rented::renew(w, m)),
            _ => Value::Null,
        },
        set: None,
    },
];

static SERVER_MOBILES_ETHEREALMOUNT_21: &[PropDef] = &[
    PropDef {
        name: "Steps",
        kind: Kind::Plain(PropType::Numeric(Numeric::Int)),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => w
                .item(i)
                .and_then(|o| o.ext.get::<crate::classes::ethereal::EtherealData>())
                .map_or(Value::Null, |d| {
                    let x = &d.steps;
                    Value::Int(i64::from(*x))
                }),
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.i64() as i32;
            if let Some(d) = w
                .item_mut(i)
                .and_then(|o| o.ext.get_mut::<crate::classes::ethereal::EtherealData>())
            {
                if d.steps != n {
                    d.steps = n.clamp(0, crate::classes::ethereal::STEPS_MAX);
                }
            };
            Ok(())
        }),
    },
    PropDef {
        name: "Rider",
        kind: Kind::Plain(PropType::EntityOf("Mobile")),
        read: GM,
        write: GM,
        get: |w, e| match e {
            EntityId::Item(i) => {
                Value::Entity(crate::classes::ethereal::rider(w, i).map(EntityId::Mobile))
            }
            _ => Value::Null,
        },
        set: Some(|w, e, v| {
            let EntityId::Item(i) = e else {
                return Err(NOT_HERE.to_string());
            };
            let n = v.mobile();
            crate::classes::ethereal::set_rider(w, i, n);
            Ok(())
        }),
    },
];

static SERVER_MOBILES_BASEESCORTABLE: &[PropDef] = &[PropDef {
    name: "Destination",
    kind: STR,
    read: GM,
    write: GM,
    get: |w, e| match e {
        EntityId::Mobile(m) => {
            crate::npcs::townfolk::destination(w, m).map_or(Value::Null, Value::Str)
        }
        _ => Value::Null,
    },
    set: Some(|w, e, v| {
        let EntityId::Mobile(m) = e else {
            return Err(NOT_HERE.to_string());
        };
        let n = v.string();
        crate::npcs::townfolk::set_destination(w, m, n);
        Ok(())
    }),
}];

pub fn configure(w: &mut World) {
    props::add(
        w,
        "Server.Items.VendorRentalContract",
        SERVER_ITEMS_VENDORRENTALCONTRACT,
    );
    props::add(
        w,
        "Server.Mobiles.EtherealMount",
        SERVER_MOBILES_ETHEREALMOUNT,
    );
    props::add(w, "Server.Mobiles.BaseHealer", SERVER_MOBILES_BASEHEALER);
    props::add(w, "Server.Mobiles.BaseHire", SERVER_MOBILES_BASEHIRE);
    props::add(
        w,
        "Server.Mobiles.PlayerVendor",
        SERVER_MOBILES_PLAYERVENDOR,
    );
    props::add(
        w,
        "Server.Mobiles.PlayerVendorPlaceholder",
        SERVER_MOBILES_PLAYERVENDORPLACEHOLDER,
    );
    props::add(
        w,
        "Server.Mobiles.RentedVendor",
        SERVER_MOBILES_RENTEDVENDOR,
    );
    props::add(
        w,
        "Server.Mobiles.PlayerBarkeeper",
        SERVER_MOBILES_PLAYERBARKEEPER,
    );
    props::add(w, "Server.Mobiles.Cow", SERVER_MOBILES_COW);
    props::add(w, "Server.Mobiles.Sheep", SERVER_MOBILES_SHEEP);
    props::add(
        w,
        "Server.Mobiles.WaterElemental",
        SERVER_MOBILES_WATERELEMENTAL,
    );
    props::add(w, "Server.Mobiles.SwampDragon", SERVER_MOBILES_SWAMPDRAGON);
    props::add(w, "Server.Mobiles.PlagueBeast", SERVER_MOBILES_PLAGUEBEAST);
    props::add(
        w,
        "Server.Mobiles.PlagueBeastLord",
        SERVER_MOBILES_PLAGUEBEASTLORD,
    );
    props::add(w, "Server.Mobiles.PlagueSpawn", SERVER_MOBILES_PLAGUESPAWN);
    props::add(w, "Server.Mobiles.StainedOoze", SERVER_MOBILES_STAINEDOOZE);
    props::add(w, "Server.Mobiles.Changeling", SERVER_MOBILES_CHANGELING);
    props::add(
        w,
        "Server.Mobiles.HarrowerTentacles",
        SERVER_MOBILES_HARROWERTENTACLES,
    );
    props::add(w, "Server.Mobiles.ArcherGuard", SERVER_MOBILES_ARCHERGUARD);
    props::add(w, "Server.Mobiles.WarriorGuard", SERVER_MOBILES_ARCHERGUARD);
    props::add(w, "Server.Mobiles.BaseHire", SERVER_MOBILES_BASEHIRE_19);
    props::add(
        w,
        "Server.Mobiles.RentedVendor",
        SERVER_MOBILES_RENTEDVENDOR_20,
    );
    props::add(
        w,
        "Server.Mobiles.EtherealMount",
        SERVER_MOBILES_ETHEREALMOUNT_21,
    );
    props::add(
        w,
        "Server.Mobiles.BaseEscortable",
        SERVER_MOBILES_BASEESCORTABLE,
    );
}
