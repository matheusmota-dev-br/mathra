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


def test_display_and_strict():
    tex = r"\sum_{n=1}^{\infty} x_n"
    assert mathra.tex_to_svg(tex, display=False) != mathra.tex_to_svg(tex)
    assert mathra.tex_to_svg(r"\vertA\vert").startswith("<svg")
    with pytest.raises(ValueError):
        mathra.tex_to_svg(r"\vertA\vert", strict=True)


def test_render_returns_every_output():
    r = mathra.render(r"\frac{3}{4}", locale="fr")
    assert r["speech"] == "trois quarts"
    assert r["svg"].startswith("<svg")
    assert r["tree"]["type"] == "row" or "type" in r["tree"]
    assert r["warnings"] == []
    broken = mathra.render(r"\vertA\vert")
    assert broken["warnings"][0]["rule"] == "E-REC-1"


def test_locales():
    langs = mathra.locales()
    assert len(langs) == 16
    assert langs[0] == {"code": "en", "name": "English"}
    assert {"code": "ja", "name": "日本語"} in langs


def test_core_version():
    assert mathra.core_version.count(".") == 2
