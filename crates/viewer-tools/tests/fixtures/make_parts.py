# Builds parts.FCStd, the FreeCAD adapter's test fixture. Run it with
# freecadcmd, the output path in FC_OUT:
#
#   FC_OUT=parts.FCStd freecadcmd make_parts.py
#
# From WSL with a Windows FreeCAD, give Windows paths and pass FC_OUT through:
#
#   FC_OUT="$(wslpath -w .)\\parts.FCStd" WSLENV=FC_OUT \
#     "/mnt/c/Program Files/FreeCAD 1.1/bin/freecadcmd.exe" "$(wslpath -w make_parts.py)"
import os

import FreeCAD as App
import Part  # noqa: F401  (registers Part::Cylinder)
import PartDesign  # noqa: F401  (registers PartDesign::Body)

doc = App.newDocument("parts")

# Body: a PartDesign body holding a 100 mm cube, 1.0e6 mm^3.
body = doc.addObject("PartDesign::Body", "Body")
cube = doc.addObject("PartDesign::AdditiveBox", "Cube")
body.addObject(cube)
cube.Length = cube.Width = cube.Height = 100

# Assembly.Link.: an App::Part holding an App::Link to a cylinder of radius
# 10 mm and height 100 mm, 10000 pi mm^3.
cylinder = doc.addObject("Part::Cylinder", "Cylinder")
cylinder.Radius = 10
cylinder.Height = 100
assembly = doc.addObject("App::Part", "Assembly")
link = doc.addObject("App::Link", "Link")
link.LinkedObject = cylinder
assembly.addObject(link)

doc.recompute()
print("Body", body.Shape.Volume, "Assembly.Link.", assembly.getSubObject("Link.").Volume)
doc.saveAs(os.environ["FC_OUT"])
