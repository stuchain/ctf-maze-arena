"""Focused real-browser verification for the Phase 6 Algorithm Race experience."""

from pathlib import Path
from playwright.sync_api import sync_playwright


BASE_URL = "http://127.0.0.1:3000"
OUTPUT = Path(__file__).parents[1] / "web" / "test-results" / "phase06"


def main() -> None:
    OUTPUT.mkdir(parents=True, exist_ok=True)
    browser_errors: list[str] = []
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(headless=True)
        page = browser.new_page(viewport={"width": 1440, "height": 1000})
        page.on("console", lambda message: browser_errors.append(message.text) if message.type == "error" else None)
        page.on("pageerror", lambda error: browser_errors.append(str(error)))
        page.goto(BASE_URL)
        page.wait_for_load_state("networkidle")
        page.get_by_label("Width").fill("20")
        page.get_by_label("Height").fill("20")
        page.get_by_label("Seed").fill("606")
        page.get_by_label("Maze Features").select_option("keys")
        page.get_by_role("button", name="Generate Maze").click()
        page.get_by_test_id("maze-grid").wait_for()
        page.get_by_role("button", name="Algorithm Race").click()
        page.get_by_label("DP KEYS").check()
        page.get_by_role("button", name="Side by Side").click()
        page.get_by_test_id("race-button").click()
        race = page.get_by_test_id("race-experience")
        race.get_by_text("Analysis Ready").wait_for(timeout=60_000)
        assert race.locator(".race-stage").count() == 4
        assert race.get_by_role("button", name="Zoom in").count() == 4
        assert race.get_by_role("columnheader", name="Peak frontier").is_visible()
        assert page.evaluate("document.documentElement.scrollWidth <= document.documentElement.clientWidth")
        page.screenshot(path=OUTPUT / "desktop-race.png", full_page=True)

        page.set_viewport_size({"width": 390, "height": 844})
        page.wait_for_timeout(100)
        visible_stages = race.locator(".race-stage:visible").count()
        assert visible_stages == 1
        assert page.evaluate("document.documentElement.scrollWidth <= document.documentElement.clientWidth")
        page.screenshot(path=OUTPUT / "mobile-race.png", full_page=True)
        browser.close()

    if browser_errors:
        raise AssertionError(f"Browser errors observed: {browser_errors}")
    print("Phase 6 UI verification passed: desktop, mobile, console, and overflow checks")


if __name__ == "__main__":
    main()
