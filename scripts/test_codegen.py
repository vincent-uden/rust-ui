from codegen import RustSimpleEnum


def test_sprite_enum():
    sprite_key = RustSimpleEnum(
        "Icon",
        [
            "Angle",
            "Coincident",
            "Colinear",
            "Distance",
        ],
        # TODO: From<&str>
        [
            "Hash",
            "Clone",
            "FromStr",
            "PartialEq",
            "Eq",
            "Debug",
            "Default",
        ],
    )

    expected = """#[derive(Hash, Clone, FromStr, PartialEq, Eq, Debug, Default)]
pub enum Icon {
    Angle,
    Coincident,
    Colinear,
    Distance,
}"""

    assert sprite_key.to_string() == expected
