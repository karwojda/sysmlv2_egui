# PROTOTYPE: builds model/cad/engine.FCStd. Run: freecadcmd.exe make_cad.py (path passed via env FC_OUT)
import os, FreeCAD as App, Part
out = os.environ["FC_OUT"]
doc = App.newDocument("engine")
box = doc.addObject("Part::Box", "Engine")          # 100 x 100 x 100 mm = 1.0e6 mm^3
box.Length = box.Width = box.Height = 100
gb = doc.addObject("Part::Cylinder", "Gearbox")     # r 40, h 50 mm
gb.Radius = 40; gb.Height = 50
asm = doc.addObject("App::Part", "Assembly")
link = doc.addObject("App::Link", "Link")
link.LinkedObject = gb
asm.addObject(link)
doc.recompute()
doc.saveAs(out)
print("saved", out)
