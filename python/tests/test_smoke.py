from bean_format import format_text


def test_round_trip_open_account() -> None:
    sample = "2010-01-01 open Assets:Cash\n"
    assert format_text(sample) == sample


if __name__ == "__main__":
    test_round_trip_open_account()
