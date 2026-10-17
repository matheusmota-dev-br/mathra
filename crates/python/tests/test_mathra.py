"""The Python package, used the way someone who installed it would."""

import json

import pytest

import mathra


def test_svg_with_default_locale():
    svg = mathra.tex_to_svg(r"\frac{1}{2}")
    assert svg.startswith("<svg")
    assert "1 over 2" in svg


def test_svg_in_portuguese():
    assert "3,14" in mathra.tex_to_svg("3,14", locale="pt-BR")
    assert "um meio" in mathra.tex_to_svg(r"\frac{1}{2}", locale="pt-BR")


def test_tree_is_json():
    tree = mathra.tex_to_tree("x^2")
    assert json.loads(tree)["type"] == "row"
    assert '"scripts"' in tree


def test_tree_in_portuguese():
    assert json.loads(mathra.tex_to_tree("x", "pt-BR"))


def test_unknown_locale():
    with pytest.raises(ValueError, match="unknown locale"):
        mathra.tex_to_svg("x", locale="xx")
    with pytest.raises(ValueError):
        mathra.tex_to_tree("x", locale="xx")


def test_conversion_error_becomes_value_error():
    # the default mode tolerates almost everything; nesting past the limit is an error in both modes
    deep = "{" * 100 + "x" + "}" * 100
    with pytest.raises(ValueError, match="nesting"):
        mathra.tex_to_svg(deep)
    with pytest.raises(ValueError):
        mathra.tex_to_tree(deep)


def test_version():
    assert mathra.__version__.count(".") == 2
