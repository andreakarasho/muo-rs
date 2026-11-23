item("BaseBoard", |c| {
    c.weight = 5.0;
});

item("Dices", |c| {
    c.weight = 1.0;
    c.on_create(|i| {
        i.item_id = 0xFA7;
    });
});

item("PieceWhiteKing", |c| {
    c.name = "white king";
    c.on_create(|i| {
        i.item_id = 0x3587;
    });
});

item("PieceBlackKing", |c| {
    c.name = "black king";
    c.on_create(|i| {
        i.item_id = 0x358E;
    });
});

item("PieceWhiteQueen", |c| {
    c.name = "white queen";
    c.on_create(|i| {
        i.item_id = 0x358A;
    });
});

item("PieceBlackQueen", |c| {
    c.name = "black queen";
    c.on_create(|i| {
        i.item_id = 0x3591;
    });
});

item("PieceWhiteRook", |c| {
    c.name = "white rook";
    c.on_create(|i| {
        i.item_id = 0x3586;
    });
});

item("PieceBlackRook", |c| {
    c.name = "black rook";
    c.on_create(|i| {
        i.item_id = 0x358D;
    });
});

item("PieceWhiteBishop", |c| {
    c.name = "white bishop";
    c.on_create(|i| {
        i.item_id = 0x3585;
    });
});

item("PieceBlackBishop", |c| {
    c.name = "black bishop";
    c.on_create(|i| {
        i.item_id = 0x358C;
    });
});

item("PieceWhiteKnight", |c| {
    c.name = "white knight";
    c.on_create(|i| {
        i.item_id = 0x3588;
    });
});

item("PieceBlackKnight", |c| {
    c.name = "black knight";
    c.on_create(|i| {
        i.item_id = 0x358F;
    });
});

item("PieceWhitePawn", |c| {
    c.name = "white pawn";
    c.on_create(|i| {
        i.item_id = 0x3589;
    });
});

item("PieceBlackPawn", |c| {
    c.name = "black pawn";
    c.on_create(|i| {
        i.item_id = 0x3590;
    });
});

item("PieceWhiteChecker", |c| {
    c.name = "white checker";
    c.on_create(|i| {
        i.item_id = 0x3584;
    });
});

item("PieceBlackChecker", |c| {
    c.name = "black checker";
    c.on_create(|i| {
        i.item_id = 0x358B;
    });
});

item("Backgammon", |c| {
    c.board_pieces = [
        ["Server.Items.PieceWhiteChecker", 42, 6],
        ["Server.Items.PieceBlackChecker", 42, 119],
        ["Server.Items.PieceBlackChecker", 142, 6],
        ["Server.Items.PieceWhiteChecker", 142, 119],
        ["Server.Items.PieceWhiteChecker", 42, 23],
        ["Server.Items.PieceBlackChecker", 42, 136],
        ["Server.Items.PieceBlackChecker", 142, 23],
        ["Server.Items.PieceWhiteChecker", 142, 136],
        ["Server.Items.PieceWhiteChecker", 42, 40],
        ["Server.Items.PieceBlackChecker", 42, 153],
        ["Server.Items.PieceBlackChecker", 142, 40],
        ["Server.Items.PieceWhiteChecker", 142, 153],
        ["Server.Items.PieceWhiteChecker", 42, 57],
        ["Server.Items.PieceBlackChecker", 42, 170],
        ["Server.Items.PieceBlackChecker", 142, 57],
        ["Server.Items.PieceWhiteChecker", 142, 170],
        ["Server.Items.PieceWhiteChecker", 42, 74],
        ["Server.Items.PieceBlackChecker", 42, 187],
        ["Server.Items.PieceBlackChecker", 142, 74],
        ["Server.Items.PieceWhiteChecker", 142, 187],
        ["Server.Items.PieceBlackChecker", 108, 6],
        ["Server.Items.PieceWhiteChecker", 108, 153],
        ["Server.Items.PieceBlackChecker", 108, 23],
        ["Server.Items.PieceWhiteChecker", 108, 170],
        ["Server.Items.PieceBlackChecker", 108, 40],
        ["Server.Items.PieceWhiteChecker", 108, 187],
        ["Server.Items.PieceWhiteChecker", 223, 6],
        ["Server.Items.PieceBlackChecker", 223, 170],
        ["Server.Items.PieceWhiteChecker", 223, 23],
        ["Server.Items.PieceBlackChecker", 223, 187],
    ];
    c.on_create(|i| {
        i.item_id = 0xE1C;
    });
});

item("CheckerBoard", |c| {
    c.label = 1016449;
    c.board_pieces = [
        ["Server.Items.PieceWhiteChecker", 45, 25],
        ["Server.Items.PieceWhiteChecker", 70, 50],
        ["Server.Items.PieceWhiteChecker", 45, 75],
        ["Server.Items.PieceBlackChecker", 70, 150],
        ["Server.Items.PieceBlackChecker", 45, 175],
        ["Server.Items.PieceBlackChecker", 70, 200],
        ["Server.Items.PieceWhiteChecker", 95, 25],
        ["Server.Items.PieceWhiteChecker", 120, 50],
        ["Server.Items.PieceWhiteChecker", 95, 75],
        ["Server.Items.PieceBlackChecker", 120, 150],
        ["Server.Items.PieceBlackChecker", 95, 175],
        ["Server.Items.PieceBlackChecker", 120, 200],
        ["Server.Items.PieceWhiteChecker", 145, 25],
        ["Server.Items.PieceWhiteChecker", 170, 50],
        ["Server.Items.PieceWhiteChecker", 145, 75],
        ["Server.Items.PieceBlackChecker", 170, 150],
        ["Server.Items.PieceBlackChecker", 145, 175],
        ["Server.Items.PieceBlackChecker", 170, 200],
        ["Server.Items.PieceWhiteChecker", 195, 25],
        ["Server.Items.PieceWhiteChecker", 220, 50],
        ["Server.Items.PieceWhiteChecker", 195, 75],
        ["Server.Items.PieceBlackChecker", 220, 150],
        ["Server.Items.PieceBlackChecker", 195, 175],
        ["Server.Items.PieceBlackChecker", 220, 200],
    ];
    c.on_create(|i| {
        i.item_id = 0xFA6;
    });
});

item("Chessboard", |c| {
    c.label = 1016450;
    c.board_pieces = [
        ["Server.Items.PieceBlackPawn", 67, 17],
        ["Server.Items.PieceWhitePawn", 192, 17],
        ["Server.Items.PieceBlackPawn", 67, 42],
        ["Server.Items.PieceWhitePawn", 192, 42],
        ["Server.Items.PieceBlackPawn", 67, 67],
        ["Server.Items.PieceWhitePawn", 192, 67],
        ["Server.Items.PieceBlackPawn", 67, 92],
        ["Server.Items.PieceWhitePawn", 192, 92],
        ["Server.Items.PieceBlackPawn", 67, 117],
        ["Server.Items.PieceWhitePawn", 192, 117],
        ["Server.Items.PieceBlackPawn", 67, 142],
        ["Server.Items.PieceWhitePawn", 192, 142],
        ["Server.Items.PieceBlackPawn", 67, 167],
        ["Server.Items.PieceWhitePawn", 192, 167],
        ["Server.Items.PieceBlackPawn", 67, 192],
        ["Server.Items.PieceWhitePawn", 192, 192],
        ["Server.Items.PieceBlackRook", 42, 5],
        ["Server.Items.PieceBlackRook", 42, 180],
        ["Server.Items.PieceWhiteRook", 216, 5],
        ["Server.Items.PieceWhiteRook", 216, 180],
        ["Server.Items.PieceBlackKnight", 42, 30],
        ["Server.Items.PieceBlackKnight", 42, 155],
        ["Server.Items.PieceWhiteKnight", 216, 30],
        ["Server.Items.PieceWhiteKnight", 216, 155],
        ["Server.Items.PieceBlackBishop", 42, 55],
        ["Server.Items.PieceBlackBishop", 42, 130],
        ["Server.Items.PieceWhiteBishop", 216, 55],
        ["Server.Items.PieceWhiteBishop", 216, 130],
        ["Server.Items.PieceBlackQueen", 42, 105],
        ["Server.Items.PieceWhiteQueen", 216, 105],
        ["Server.Items.PieceBlackKing", 42, 80],
        ["Server.Items.PieceWhiteKing", 216, 80],
    ];
    c.on_create(|i| {
        i.item_id = 0xFA6;
    });
});
