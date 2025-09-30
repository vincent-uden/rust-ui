#version 330
// uniform uint gDrawIndex;
uniform uint gObjectIndex;
uniform vec4 color;

out vec4 FragColor;

void main() {
    FragColor = vec4(float(gObjectIndex) / 255.0, 1.0, 1.0, 1.0);
}
