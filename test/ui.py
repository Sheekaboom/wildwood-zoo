"""Wildwood Zoo browser UI test suite (desktop + mobile). Usage: python3 test/ui.py"""
import asyncio, json, sys
from playwright.async_api import async_playwright

import pathlib, os
URL = (pathlib.Path(__file__).resolve().parent.parent / "dist" / "index.html").as_uri()
CHROME = os.environ.get("CHROME")  # optional path to a Chrome binary
passed, failed, failures = 0, 0, []

def ok(cond, name, extra=None):
    global passed, failed
    if cond:
        passed += 1
    else:
        failed += 1
        failures.append(name + (f"  → {extra}" if extra is not None else ""))

async def scr(pg, x, y):
    """Screen position (CSS px) of the centre of world tile (x, y)."""
    return await pg.evaluate(f"proj({x}+0.5,{y}+0.5)")

async def drag(pg, a, b, steps=12):
    await pg.mouse.move(*a); await pg.mouse.down(); await pg.mouse.move(*b, steps=steps); await pg.mouse.up()

async def stats(pg):
    return await pg.evaluate("J(X.stats())")

async def desktop(b):
    ctx = await b.new_context(viewport={"width": 1366, "height": 820})
    pg = await ctx.new_page()
    errs = []
    pg.on("pageerror", lambda e: errs.append(str(e)))
    pg.on("console", lambda m: errs.append(m.text) if m.type == "error" else None)
    await pg.goto(URL)
    await pg.wait_for_timeout(900)
    # ---- first run ----
    ok(await pg.is_visible("#modal.open"), "first run shows welcome + size picker")
    await pg.click('#modalBox button:has-text("Medium")')
    await pg.wait_for_timeout(300)
    s = await stats(pg)
    ok(s["own"][2] - s["own"][0] == 80, "clicking Medium starts an 80×80 zoo", s["own"])
    ok(not await pg.is_visible("#modal.open"), "welcome closes after choosing")
    await pg.evaluate("setSpeed(0)")
    own = s["own"]; x0, y1 = own[0], own[3]
    # clear the random scenery from the test area so clicks land on empty ground
    await pg.evaluate(f"for(let y={y1-29};y<={y1-17};y++) for(let x={x0+7};x<={x0+21};x++) {{ if (O[y*W+x]) X.bulldoze(x+0.5,y+0.5); X.paint_terrain(x,y,0,0); }}")
    # ---- every toolbar category opens ----
    cats = await pg.evaluate("CATS.map(c=>c.id)")
    for c in cats:
        if c in ("walk", "view"):
            continue
        await pg.click(f'#tools .btn[data-cat="{c}"]')
        await pg.wait_for_timeout(80)
        n = await pg.evaluate("document.querySelectorAll('#trayItems .item').length")
        expect = {"select": 0, "exhibit": 3, "animals": 12, "terrain": 8, "foliage": 13, "items": 3, "guest": 7, "scenery": 3, "path": 1, "staff": 2, "bulldoze": 0, "land": 3}[c]
        ok(n == expect, f"category '{c}' shows {expect} items", n)
    # ---- build an exhibit with a real mouse drag ----
    await pg.evaluate(f"view.cx={x0+14}; view.cy={y1-22}; view.zoom=1; view.rot=0; buildOrder();")
    await pg.click('#tools .btn[data-cat="exhibit"]')
    await pg.click("#trayItems .item >> nth=1")  # chain-link
    cash0 = (await stats(pg))["cash"]
    await drag(pg, await scr(pg, x0 + 8, y1 - 28), await scr(pg, x0 + 20, y1 - 18))
    await pg.wait_for_timeout(150)
    t = await pg.evaluate(f"JSON.parse(outStr(X.info_tile({x0+14}.5,{y1-23}.5)))")
    ok(t["enclosure"] and t["enclosure"]["area"] == 13 * 11, "mouse-drag builds a 13×11 exhibit", t["enclosure"] and t["enclosure"]["area"])
    ok((await stats(pg))["cash"] < cash0, "exhibit cost deducted")
    # ---- paint terrain by drag ----
    await pg.click('#tools .btn[data-cat="terrain"]')
    await pg.click("#trayItems .item >> nth=1")  # savanna
    await pg.click('#trayTitle [data-b="2"]')
    for row in range(y1 - 27, y1 - 18, 2):
        await drag(pg, await scr(pg, x0 + 9, row), await scr(pg, x0 + 19, row), steps=20)
    t = await pg.evaluate(f"JSON.parse(outStr(X.info_tile({x0+14}.5,{y1-23}.5)))")
    ok(t["enclosure"]["terrain"][1] > 0.6, "drag-painting covers the exhibit with savanna", t["enclosure"]["terrain"])
    # ---- place foliage & items by click ----
    await pg.click('#tools .btn[data-cat="foliage"]')
    await pg.click("#trayItems .item >> nth=2")  # acacia
    for (dx, dy) in [(10, -27), (13, -25), (17, -26), (11, -21), (18, -20)]:
        await pg.mouse.click(*await scr(pg, x0 + dx, y1 + dy))
    await pg.click('#tools .btn[data-cat="items"]')
    for i, (dx, dy) in enumerate([(15, -24), (12, -19), (19, -23)]):
        await pg.click(f"#trayItems .item >> nth={i}")
        await pg.mouse.click(*await scr(pg, x0 + dx, y1 + dy))
    t = await pg.evaluate(f"JSON.parse(outStr(X.info_tile({x0+14}.5,{y1-23}.5)))")
    ok(t["enclosure"]["water"], "water trough placed by click gives the exhibit water")
    # ---- adopt zebras: hover tooltip shows suitability ----
    await pg.click('#tools .btn[data-cat="animals"]')
    await pg.click("#trayItems .item >> nth=1")
    await pg.mouse.move(*await scr(pg, x0 + 14, y1 - 23)); await pg.wait_for_timeout(100)
    tip = await pg.inner_text("#cursorTip")
    ok("suitability" in tip, "hover tooltip shows habitat suitability", tip)
    for i in range(6):
        if i == 3:
            await pg.click("#sexF")
        await pg.mouse.click(*await scr(pg, x0 + 11 + i, y1 - 23))
    s = await stats(pg)
    ok(s["animals"] == 6, "6 zebras adopted by clicking", s["animals"])
    await pg.wait_for_timeout(400)
    ok(await pg.is_visible("#info.open") and "Plains Zebra" in await pg.inner_text("#info"), "info panel opens on the new animal")
    # ---- staff + buildings along the path ----
    await pg.click('#tools .btn[data-cat="staff"]')
    await pg.click("#trayItems .item >> nth=0"); await pg.click("#trayItems .item >> nth=1")
    s = await stats(pg); ok(s["keepers"] == 1 and s["maint"] == 1, "hiring staff from the menu", [s["keepers"], s["maint"]])
    await pg.click('#tools .btn[data-cat="guest"]')
    await pg.evaluate("view.cx=64; view.cy=112; buildOrder()")
    for i, (x, y) in enumerate([(60, 113), (62, 113), (66, 113), (68, 115)]):
        await pg.click(f"#trayItems .item >> nth={i}")
        await pg.mouse.click(*await scr(pg, x, y))
    objs = await pg.evaluate("[O[113*W+60],O[113*W+62],O[113*W+66],O[115*W+68]]")
    ok(objs == [17, 18, 19, 20], "food, soda, restroom, bench placed", objs)
    # ---- select tool: tap entity, drag to pan ----
    await pg.click('#tools .btn[data-cat="select"]')
    cx0 = await pg.evaluate("view.cx")
    await drag(pg, (700, 400), (500, 300))
    ok(abs(await pg.evaluate("view.cx") - cx0) > 1, "dragging with Select pans the camera")
    # ---- speed controls ----
    await pg.evaluate("setSpeed(0)"); d0 = (await stats(pg))["date"]; tm0 = await pg.evaluate("J(X.stats()).dayFrac")
    await pg.wait_for_timeout(800)
    ok(await pg.evaluate("J(X.stats()).dayFrac") == tm0, "pause stops time")
    await pg.click('#speed .btn[data-s="4"]'); await pg.wait_for_timeout(800)
    ok(await pg.evaluate("J(X.stats()).dayFrac") != tm0, "fastest speed runs time")
    await pg.wait_for_timeout(6000)
    s = await stats(pg); ok(s["guests"] > 0, "guests arrive while the game runs", s["guests"])
    # ---- rename a guest through the dialog ----
    gid = await pg.evaluate("(()=>{const n=X.entities();const e=new Float32Array(X.memory.buffer,X.entities_ptr(),n*10);for(let i=0;i<n;i++) if(e[i*10]==2) return e[i*10+7]; return -1})()")
    if gid > 0:
        await pg.evaluate(f"selected={{kind:'ent',id:{gid}}}; refreshInfo(true)")
        await pg.click("#info .btn >> text=✏️")
        await pg.fill("#rnI", "Mr. Orange"); await pg.press("#rnI", "Enter")
        await pg.wait_for_timeout(300)
        ok((await stats(pg))["tint"] == 3, "renaming a guest to Mr. Orange via the dialog fires the cheat")
        ok("Mr. Orange" in await pg.inner_text("#info"), "new name shown in info panel")
    else:
        ok(False, "a guest exists to rename")
    # ---- rename exhibit via dialog ----
    await pg.evaluate(f"selected={{kind:'tile',x:{x0+14},y:{y1-23},fx:{x0+14}.5,fy:{y1-23}.5}}; refreshInfo(true)")
    await pg.click("#info .btn >> text=✏️")
    await pg.fill("#rnI", "Savanna Sunrise"); await pg.press("#rnI", "Enter"); await pg.wait_for_timeout(300)
    ok("Savanna Sunrise" in await pg.inner_text("#info h3"), "exhibit renamed via dialog")
    # ---- terminal ----
    await pg.click("#termBtn"); await pg.wait_for_timeout(150)
    ok(await pg.is_visible("#term.open"), ">_ button opens the terminal")
    focused = await pg.evaluate("document.activeElement.id")
    ok(focused == "termIn", "terminal input is focused", focused)
    for cmd in ["help", "cheats", "status", "Xanadu", "reset"]:
        await pg.keyboard.type(cmd); await pg.keyboard.press("Enter")
    out = await pg.inner_text("#termOut")
    ok("WILDWOOD ZOO TERMINAL" in out and "Copralite Corner" in out and "Zoo size: 80 x 80" in out, "terminal help/cheats/status output")
    ok("Cheat activated: Xanadu" in out, "terminal cheat fires (renames selected exhibit)")
    ok((await stats(pg))["unlocked"] & 4096, "Unicorn unlocked")
    await pg.keyboard.press("ArrowUp"); ok(await pg.input_value("#termIn") == "reset", "terminal history (↑)")
    await pg.fill("#termIn", "clear"); await pg.keyboard.press("Enter")
    ok((await pg.inner_text("#termOut")).strip() == "", "clear empties the terminal")
    cash_before = (await stats(pg))["cash"]
    await pg.keyboard.type("$")
    ok(abs((await stats(pg))["cash"] - cash_before) < 50, "typing $ inside the terminal does NOT trigger the money cheat")
    await pg.keyboard.press("Escape"); await pg.wait_for_timeout(100)
    ok(not await pg.is_visible("#term.open"), "Escape closes the terminal")
    await pg.keyboard.press("Backquote"); await pg.wait_for_timeout(100)
    ok(await pg.is_visible("#term.open"), "~ / ` key opens the terminal")
    await pg.keyboard.press("Backquote"); await pg.wait_for_timeout(100)
    ok(not await pg.is_visible("#term.open"), "~ / ` key closes it again")
    # unicorn now in animal tray
    await pg.click('#tools .btn[data-cat="animals"]'); await pg.wait_for_timeout(100)
    ok(await pg.evaluate("document.querySelectorAll('#trayItems .item').length") == 13, "Unicorn appears in the Animals menu after unlock")
    await pg.click('#tools .btn[data-cat="select"]')
    # ---- Shift+$ ----
    await pg.mouse.click(683, 600)
    c0 = (await stats(pg))["cash"]; await pg.keyboard.press("Shift+Digit4"); await pg.wait_for_timeout(100)
    ok(round((await stats(pg))["cash"] - c0) >= 9900, "Shift+$ gives $10,000")
    # ---- rotation & zoom consistency ----
    for r in range(4):
        await pg.keyboard.press("e"); await pg.wait_for_timeout(60)
        rt = await pg.evaluate("(()=>{const p=proj(70.3,100.7); const u=unproj(p[0],p[1]); return [view.rot, Math.abs(u[0]-70.3)+Math.abs(u[1]-100.7)]})()")
        ok(rt[1] < 1e-6, f"rotation {rt[0]}: screen↔world mapping round-trips", rt)
    z0 = await pg.evaluate("view.zoom"); await pg.mouse.move(683, 410); await pg.mouse.wheel(0, -300); await pg.wait_for_timeout(100)
    ok(await pg.evaluate("view.zoom") > z0, "mouse wheel zooms in")
    # ---- menus/modals ----
    await pg.click("#stCash"); await pg.wait_for_timeout(100)
    ok("Finances" in await pg.inner_text("#modalBox"), "clicking cash opens Finances")
    await pg.evaluate("const r=document.querySelector('#modalBox input[type=range]'); r.value=35; r.dispatchEvent(new Event('input'))")
    ok((await stats(pg))["admission"] == 35, "admission slider sets price")
    await pg.evaluate("closeModal()")
    for sel, txt in [("#stFame", "Zoo Status"), ("#stDate", "Goals")]:
        await pg.click(sel); await pg.wait_for_timeout(80)
        ok(txt in await pg.inner_text("#modalBox"), f"{sel} opens {txt}"); await pg.evaluate("closeModal()")
    await pg.click("#menuBtn"); await pg.wait_for_timeout(80)
    menu = await pg.inner_text("#modalBox")
    ok(all(k in menu for k in ["Save Zoo", "Load Zoo", "Export", "Import", "Terminal", "How to Play", "New Zoo"]), "main menu has all entries")
    # ---- buy land through the UI ----
    await pg.evaluate("closeModal()")
    await pg.click('#tools .btn[data-cat="land"]')
    w0 = (await stats(pg))["own"]
    await pg.click("#trayItems .item >> nth=2")  # east
    await pg.wait_for_timeout(200)
    w1 = (await stats(pg))["own"]
    ok(w1[2] == w0[2] + 8, "Buy Land → Expand East adds 8 columns", [w0, w1])
    ok("88 × 80" in await pg.inner_text("#trayTitle"), "Land menu shows new size")
    await pg.click('#tools .btn[data-cat="select"]')
    # ---- save to browser, reload page, auto-restore ----
    await pg.click("#menuBtn"); await pg.click("text=💾 Save Zoo"); await pg.wait_for_timeout(200)
    before = await stats(pg)
    await pg.reload(); await pg.wait_for_timeout(1200)
    after = await stats(pg)
    ok(after["animals"] == before["animals"] and after["own"] == before["own"] and after["unlocked"] == before["unlocked"], "Save Zoo + page reload restores the zoo", [before["animals"], after["animals"], after["own"]])
    ok(not await pg.is_visible("#modal.open"), "no welcome screen when a save exists")
    # ---- export / new zoo / import ----
    await pg.click("#menuBtn"); await pg.click("text=⬇ Export save code"); await pg.wait_for_timeout(150)
    code = await pg.input_value("#sc")
    ok(1000 < len(code) < 4000, f"export code length {len(code)} chars", len(code))
    await pg.evaluate("closeModal()")
    await pg.click("#menuBtn"); await pg.click("text=🌱 New Zoo"); await pg.click('#modalBox button:has-text("Large")')
    await pg.wait_for_timeout(200)
    s = await stats(pg); ok(s["own"][2] - s["own"][0] == 112 and s["animals"] == 0, "New Zoo → Large starts fresh 112×112")
    await pg.click("#menuBtn"); await pg.click("text=⬆ Import save code")
    await pg.fill("#ic", code); await pg.click("#modalBox >> text=Load"); await pg.wait_for_timeout(300)
    s = await stats(pg); ok(s["animals"] == before["animals"] and s["own"] == before["own"], "Import save code restores the exported zoo", [s["animals"], s["own"]])
    await pg.click("#menuBtn"); await pg.click("text=⬆ Import save code")
    await pg.fill("#ic", "not a real save code!!"); await pg.click("#modalBox >> text=Load"); await pg.wait_for_timeout(300)
    s2 = await stats(pg); ok(s2["animals"] == s["animals"], "bad import code is rejected without harming the zoo")
    ok("could not be loaded" in await pg.inner_text("#toasts"), "bad import code shows an error toast")
    await pg.evaluate("closeModal()")
    # ---- Zoo Walk ----
    await pg.click('#tools .btn[data-cat="walk"]'); await pg.wait_for_timeout(300)
    ok(await pg.is_visible("#walkWrap.open"), "Zoo Walk opens")
    p0 = await pg.evaluate("[X.walk_x(), X.walk_y(), X.walk_a()]")
    await pg.keyboard.down("w"); await pg.wait_for_timeout(700); await pg.keyboard.up("w")
    p1 = await pg.evaluate("[X.walk_x(), X.walk_y()]")
    moved = abs(p1[0] - p0[0]) + abs(p1[1] - p0[1])
    await pg.keyboard.down("Shift"); await pg.keyboard.down("w"); await pg.wait_for_timeout(700); await pg.keyboard.up("w"); await pg.keyboard.up("Shift")
    p2 = await pg.evaluate("[X.walk_x(), X.walk_y()]")
    ran = abs(p2[0] - p1[0]) + abs(p2[1] - p1[1])
    ok(moved > 0.5, "W walks forward", moved)
    ok(ran > moved * 1.5 or ran < 0.2, "Shift+W runs faster (unless blocked)", [moved, ran])
    await pg.keyboard.down("ArrowLeft"); await pg.wait_for_timeout(300); await pg.keyboard.up("ArrowLeft")
    ok(abs(await pg.evaluate("X.walk_a()") - p0[2]) > 0.2, "arrow keys turn")
    await pg.mouse.move(600, 400); await pg.mouse.down(); await pg.mouse.move(750, 400, steps=5); await pg.mouse.up()
    nonblack = await pg.evaluate("(()=>{const c=document.getElementById('walkCanvas').getContext('2d').getImageData(0,0,document.getElementById('walkCanvas').width,document.getElementById('walkCanvas').height).data; let n=0; for(let i=0;i<c.length;i+=97) if(c[i]+c[i+1]+c[i+2]>60) n++; return n})()")
    ok(nonblack > 300, "Zoo Walk canvas is rendering", nonblack)
    hor = await pg.evaluate("(()=>{const c=document.getElementById('walkCanvas'); const d=c.getContext('2d').getImageData(0,0,c.width,c.height).data; let black=0; for(let y=0;y<c.height;y++){ let row=0; for(let x=0;x<c.width;x+=7){const i=(y*c.width+x)*4; if(d[i]+d[i+1]+d[i+2]<15) row++;} if(row>c.width/7*0.9) black++;} return black})()")
    ok(hor == 0, "no solid black rows (horizon line fixed)", hor)
    await pg.screenshot(path="test/ui_walk.png")
    await pg.click("#walkExit"); await pg.wait_for_timeout(150)
    ok(not await pg.is_visible("#walkWrap.open"), "Exit leaves Zoo Walk")
    await pg.screenshot(path="test/ui_desktop.png")
    ok(not errs, "no JavaScript errors on desktop", errs[:3])
    await ctx.close()

async def blocked_storage(b):
    ctx = await b.new_context(viewport={"width": 1280, "height": 800})
    await ctx.add_init_script("Object.defineProperty(window,'localStorage',{get(){throw new DOMException('blocked','SecurityError')}});")
    pg = await ctx.new_page(); errs = []
    pg.on("pageerror", lambda e: errs.append(str(e)))
    await pg.goto(URL); await pg.wait_for_timeout(900)
    ok(await pg.evaluate("typeof X === 'object' && X !== null"), "game boots when browser storage is blocked")
    await pg.evaluate("closeModal(); saveGame()"); await pg.wait_for_timeout(200)
    ok("Export save code" in await pg.inner_text("#toasts"), "Save explains to use Export when storage is blocked")
    await pg.evaluate("autosaveT = -1e9; uiTick(performance.now())")
    ok(not errs, "no errors with blocked storage (incl. autosave)", errs[:3])
    await ctx.close()

async def mobile(b):
    ctx = await b.new_context(viewport={"width": 390, "height": 844}, device_scale_factor=3, is_mobile=True, has_touch=True)
    pg = await ctx.new_page(); errs = []
    pg.on("pageerror", lambda e: errs.append(str(e)))
    pg.on("console", lambda m: errs.append(m.text) if m.type == "error" else None)
    await pg.goto(URL + "?new"); await pg.wait_for_timeout(900)
    await pg.tap('#modalBox button:has-text("Small")'); await pg.wait_for_timeout(200)
    ok(not await pg.is_visible("#modal.open"), "tap Small on welcome screen")
    layout = await pg.evaluate("({sw: document.documentElement.scrollWidth, iw: innerWidth, top: document.getElementById('top').getBoundingClientRect().height, tools: document.getElementById('tools').getBoundingClientRect().bottom})")
    ok(layout["sw"] <= layout["iw"], "no horizontal overflow on phone", layout)
    ok(layout["top"] < 110, "top bar fits in two rows", layout["top"])
    ok(layout["tools"] <= 844, "toolbar fully on screen", layout["tools"])
    s = await stats(pg); x0, y1 = s["own"][0], s["own"][3]
    await pg.evaluate(f"view.cx={x0+20}; view.cy={y1-20}; setSpeed(0)")
    # tap-place a tree
    await pg.tap('#tools .btn[data-cat="foliage"]'); await pg.tap("#trayItems .item >> nth=0")
    tx, ty = await scr(pg, x0 + 20, y1 - 22)
    await pg.touchscreen.tap(tx, ty); await pg.wait_for_timeout(100)
    ok(await pg.evaluate(f"O[{y1-22}*W+{x0+20}]") == 1, "tap places foliage on phone")
    # two-finger pan while a build tool is active (synthetic pointer events)
    cx0, cy0 = await pg.evaluate("[view.cx, view.cy]")
    trees0 = await pg.evaluate("Array.from(O).filter(k=>k===1).length")
    await pg.evaluate("""(()=>{const cv=document.getElementById('game'); const ev=(t,id,x,y)=>cv.dispatchEvent(new PointerEvent(t,{pointerId:id,pointerType:'touch',clientX:x,clientY:y,bubbles:true,isPrimary:id===1}));
      ev('pointerdown',1,150,400); ev('pointerdown',2,250,400);
      for(let i=1;i<=10;i++){ ev('pointermove',1,150+i*8,400+i*6); ev('pointermove',2,250+i*8,400+i*6); }
      ev('pointerup',1,230,460); ev('pointerup',2,330,460); })()""")
    cx1, cy1 = await pg.evaluate("[view.cx, view.cy]")
    trees1 = await pg.evaluate("Array.from(O).filter(k=>k===1).length")
    ok(abs(cx1 - cx0) + abs(cy1 - cy0) > 1, "two-finger drag pans on phone", [cx0, cy0, cx1, cy1])
    ok(trees1 - trees0 <= 1, "two-finger pan doesn't spray objects", trees1 - trees0)
    # pinch zoom
    z0 = await pg.evaluate("view.zoom")
    await pg.evaluate("""(()=>{const cv=document.getElementById('game'); const ev=(t,id,x,y)=>cv.dispatchEvent(new PointerEvent(t,{pointerId:id,pointerType:'touch',clientX:x,clientY:y,bubbles:true}));
      ev('pointerdown',3,180,400); ev('pointerdown',4,210,400);
      for(let i=1;i<=10;i++){ ev('pointermove',3,180-i*8,400); ev('pointermove',4,210+i*8,400); }
      ev('pointerup',3,100,400); ev('pointerup',4,290,400); })()""")
    ok(await pg.evaluate("view.zoom") > z0 * 1.5, "pinch zooms in on phone")
    # tap select an animal-free tile → info panel; terminal on phone
    await pg.tap('#tools .btn[data-cat="select"]')
    await pg.tap("#termBtn"); await pg.wait_for_timeout(150)
    box = await pg.evaluate("document.getElementById('term').getBoundingClientRect().toJSON()")
    ok(box["width"] <= 390 and box["bottom"] <= 844, "terminal fits the phone screen", box)
    await pg.fill("#termIn", "money"); await pg.press("#termIn", "Enter")
    ok("Granted $10,000" in await pg.inner_text("#termOut"), "phone users can get money via the terminal")
    await pg.tap("#termClose")
    # Zoo Walk d-pad
    await pg.tap('#tools .btn[data-cat="walk"]'); await pg.wait_for_timeout(200)
    p0 = await pg.evaluate("[X.walk_x(), X.walk_y()]")
    await pg.evaluate("""(()=>{const b=document.querySelector('#dpad [data-k=f]'); b.dispatchEvent(new PointerEvent('pointerdown',{bubbles:true,pointerType:'touch'}));})()""")
    await pg.wait_for_timeout(600)
    await pg.evaluate("""(()=>{const b=document.querySelector('#dpad [data-k=f]'); b.dispatchEvent(new PointerEvent('pointerup',{bubbles:true,pointerType:'touch'}));})()""")
    p1 = await pg.evaluate("[X.walk_x(), X.walk_y()]")
    ok(abs(p1[0] - p0[0]) + abs(p1[1] - p0[1]) > 0.4, "Zoo Walk d-pad moves on phone", [p0, p1])
    dims = await pg.evaluate("[document.getElementById('walkCanvas').width, document.getElementById('walkCanvas').height]")
    ok(dims[1] > dims[0], "Zoo Walk renders a portrait frame on a portrait phone", dims)
    await pg.evaluate(f"X.walk_set({x0}+14.5,{y1}-6.5,-Math.PI/2)"); await pg.wait_for_timeout(300)
    await pg.screenshot(path="test/ui_mobile_walk.png")
    await pg.tap("#walkExit")
    fps = await pg.evaluate("new Promise(r=>{let n=0;const t0=performance.now();(function f(){n++; if(performance.now()-t0<1500) requestAnimationFrame(f); else r(n/1.5)})()})")
    ok(fps > 30, f"phone frame rate {fps:.0f} fps (headless)", fps)
    await pg.screenshot(path="test/ui_mobile.png")
    ok(not errs, "no JavaScript errors on phone", errs[:3])
    await ctx.close()

async def main():
    async with async_playwright() as p:
        b = await p.chromium.launch(executable_path=CHROME, args=["--no-sandbox"]) if CHROME else await p.chromium.launch()
        for name, fn in [("Desktop", desktop), ("Blocked storage", blocked_storage), ("Mobile", mobile)]:
            print("##", name)
            try:
                await fn(b)
            except Exception as e:
                ok(False, f"{name} suite crashed", str(e).splitlines()[0])
        await b.close()
    print(f"\n==== {passed} passed, {failed} failed ====")
    for f in failures:
        print("FAIL:", f)
    sys.exit(1 if failed else 0)

asyncio.run(main())
