#version 330 core

out vec4 color;
in vec2 fragCoord;

uniform vec2 size; // The size on screen
uniform vec4 bgColor;
uniform vec4 traceColor;
uniform sampler2D text;
uniform float maxTraces;
uniform vec2 yLimits; // min_y, max_y

//
//                        max_y
//         ------------------------------------
//        |                                    |
//        |                                    |
//  min_x |                                    | max_x
//        |                                    |
//        |                                    |
//         ------------------------------------
//                        min_y
// x limits are determined by what is in the texture

const float lineWidth = 4.0;
const float lineHalfWidth = lineWidth / 2.0;
const float loopBound = ceil(lineHalfWidth) + 1.0;

// Params:
// - x       : x normalized UV coordinates [0.0-1.0]
// - channel : the channel or trace (0-indexed)
float height(float x, int channel) {
    return 1.0 - ((texture(text, vec2(x, channel / maxTraces)).r) - yLimits.x) / (yLimits.y - yLimits.x);
}

void main() {
    float dist = abs(height(fragCoord.x, 0) - fragCoord.y);
    float tol = 0.05;
    color = vec4(dist < tol, 0.0, 0.0, 1.0);
}
