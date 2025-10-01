#version 330
uniform uint entityId;
uniform uint sketchId;

uniform vec4 color;

out vec4 FragColor;

void main() {
    uint elow = entityId & 0xFFu;
    uint ehigh = (entityId >> 8) & 0xFFu;
    uint slow = sketchId & 0xFFu;
    uint shigh = (sketchId >> 8) & 0xFFu;
    FragColor = vec4(
        float(elow) / 255.0, 
        float(ehigh) / 255.0, 
        float(slow) / 255.0, 
        float(shigh) / 255.0
    );
}
