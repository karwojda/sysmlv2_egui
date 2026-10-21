# PROTOTYPE: answers one Tool Link query. Env FC_REQ = JSON {"file": winpath, "objects": {"<object>": ["Volume", ...]}}
# Prints one line: FCRESULT {"<object>": {"Volume": [value, "mm^3"], ...} | {"error": "..."}}
import os, json, FreeCAD as App
req = json.loads(os.environ["FC_REQ"])
out = {}
try:
    doc = App.openDocument(req["file"])
except Exception as e:
    print("FCRESULT " + json.dumps({"__file__": {"error": "cannot open %s: %s" % (req["file"], e)}})); raise SystemExit
for path, props in req["objects"].items():
    try:
        if "." in path:
            top, sub = path.split(".", 1)
            shape = doc.getObject(top).getSubObject(sub)      # assembly path, e.g. Assembly.Link.
            target = doc.getObject(top).getSubObject(sub, retType=1)
        else:
            obj = doc.getObject(path)
            if obj is None: raise LookupError("no object named %r" % path)
            shape, target = obj.Shape, obj
        r = {}
        for p in props:
            if p == "Volume": r[p] = [shape.Volume, "mm^3"]
            elif p == "Area": r[p] = [shape.Area, "mm^2"]
            elif p.startswith("CenterOfMass."):
                r[p] = [getattr(shape.CenterOfMass, p[-1]), "mm"]
            elif p == "Mass":
                mat = getattr(getattr(target, "getLinkedObject", lambda: target)(), "ShapeMaterial", None)
                name = getattr(mat, "Name", None)
                if mat is None or name in (None, "", "Default"):
                    r[p] = {"error": "material is %s" % (name or "not set")}
                else:
                    dens = App.Units.Quantity(mat.PhysicalProperties["Density"]).getValueAs("kg/mm^3")
                    r[p] = [shape.Volume * float(dens), "kg"]
            else: r[p] = {"error": "unknown property %s" % p}
        out[path] = r
    except Exception as e:
        out[path] = {"error": "%s: %s" % (type(e).__name__, e)}
print("FCRESULT " + json.dumps(out))
