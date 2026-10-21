# Reads properties of objects in one FreeCAD document; run by freecadcmd for
# the FreeCAD adapter (viewer-tools/src/freecad.rs).
#
# Request, in the environment variable SYSML_VIEWER_FREECAD_REQUEST:
#   {"file": <path>, "objects": {"<object>": ["Volume", ...]}}
# An object is a Name ("Body") or a dotted assembly path from a top-level
# object ("Assembly.Link.").
#
# Answer, one line on stdout:
#   FCRESULT {"<object>": {"Volume": [<number>, "mm^3"]} | {"error": "<reason>"}}
# with "__file__": {"error": <reason>} when the document can't be opened. A
# property's own answer may also be {"error": <reason>}.
import json
import os

import FreeCAD as App


def answer(result):
    print("FCRESULT " + json.dumps(result), flush=True)


def shape_of(doc, path):
    """The shape `path` names in `doc`; raises LookupError saying why not."""
    top, dot, sub = path.partition(".")
    obj = doc.getObject(top)
    if obj is None:
        raise LookupError("no object named %r" % top)
    if dot:
        shape = obj.getSubObject(sub)
        if shape is None:
            raise LookupError("no object at %r" % path)
        return shape
    shape = getattr(obj, "Shape", None)
    if shape is None:
        raise LookupError("object %r has no shape" % path)
    return shape


def significant(value):
    # OCCT integrates volumes numerically: a 100 mm cube comes out as
    # 999999.9999999999. Twelve significant digits is far finer than any
    # tolerance a model states and drops that noise.
    return float("%.12g" % value)


def read(shape, prop):
    if prop == "Volume":
        return [significant(shape.Volume), "mm^3"]
    return {"error": "FreeCAD has no property %s (it reads Volume)" % prop}


def main():
    request = json.loads(os.environ["SYSML_VIEWER_FREECAD_REQUEST"])
    try:
        doc = App.openDocument(request["file"])
    except Exception as e:
        answer({"__file__": {"error": "FreeCAD could not open it: %s" % e}})
        return
    result = {}
    for path, props in request["objects"].items():
        try:
            shape = shape_of(doc, path)
            result[path] = {prop: read(shape, prop) for prop in props}
        except LookupError as e:
            result[path] = {"error": str(e)}
        except Exception as e:
            result[path] = {"error": "%s: %s" % (type(e).__name__, e)}
    answer(result)


main()
