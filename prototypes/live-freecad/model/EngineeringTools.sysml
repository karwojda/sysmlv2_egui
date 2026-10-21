// PROTOTYPE copy of the library users would copy into their Model (ADR 0004).
package EngineeringTools {
    private import ScalarValues::*;

    metadata def ToolLink {
        doc /* The annotated usage is designed in `object` inside `file`, owned by Engineering Tool `tool`. */
        attribute tool : String;
        attribute file : String;
        attribute object : String;
    }

    metadata def ToolProperty {
        doc /* The annotated feature takes its Design Value from property `name` of its owner's Tool Link. */
        attribute name : String;
        attribute tool : String[0..1];
    }
}
