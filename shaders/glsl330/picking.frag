#version 330
// uniform uint gDrawIndex;
uniform uint gObjectIndex;

out vec3 color;

void main() {
    // This line was from the tutorial. Do I really need the gl_PrimitiveId? I dont think so
    // color = vec3(float(gObjectIndex), float(gDrawIndex), float(gl_PrimitiveId + 1));
     color = vec3(float(gObjectIndex), 0.0, 0.0);
}
