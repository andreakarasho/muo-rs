shop("SBBanker", |s| {
    s.sells("ContractOfEmployment", 1252, 20, 0x14F0, #{ name: "1041243", hue: 0 });
    s.when("newVendorSystem", |s| {
        s.sells("VendorRentalContract", 1252, 20, 0x14F0, #{ name: "1062332", hue: 1650 });
    });
    s.sells("CommodityDeed", 5, 20, 0x14F0, #{ name: "1047016", hue: 71 });
});
