#version 330 core

out vec4 color;
in vec2 fragCoord;

uniform vec2 size; // The size on screen
uniform vec4 bgColor;
uniform vec4 traceColor;

void main() {
    color = vec4(fragCoord.x, fragCoord.y, 0.0, 1.0);
}
