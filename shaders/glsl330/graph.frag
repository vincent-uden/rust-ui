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

const float lineWidth = 2.0;
const float lineHalfWidth = lineWidth / 2.0;
const float loopBound = ceil(lineHalfWidth) + 1.0;

// Params:
// - x       : x normalized UV coordinates [0.0-1.0]
// - channel : the channel or trace (0-indexed)
float height(float x, int channel) {
    return 1.0 - ((texture(text, vec2(x, channel / maxTraces)).r) - yLimits.x) / (yLimits.y - yLimits.x);
}

float heightSS(float x, int channel) {
    return (1.0 - ((texture(text, vec2(x / size.x, channel / maxTraces)).r) - yLimits.x) / (yLimits.y - yLimits.x)) * size.y;
}

vec2 uvToScreenSpace(vec2 coord) {
    return coord * size;
}

float distanceToLine(vec2 a, vec2 b, vec2 p) {
    float squaredLineLength = dot(b - a, b - a);
    float t = clamp(dot(p - a, b - a) / squaredLineLength, 0., 1.);
    return distance(p, a + t * (b - a));
}

void main() {
    vec2 coord = uvToScreenSpace(fragCoord);
    float dist = lineHalfWidth + 1.0;
    vec2 previousPoint = vec2(coord.x - lineHalfWidth, heightSS(coord.x - lineHalfWidth, 0));

    for (float i = -loopBound + 1.; i <= loopBound; i += 1.) {
        vec2 currentPoint = vec2(coord.x + i, heightSS(coord.x + i, 0));
        dist = min(dist, distanceToLine(previousPoint, currentPoint, coord));
        previousPoint = currentPoint;
    }

    float alpha = clamp(lineHalfWidth + 0.5 - dist, 0., 1.);
    if (coord.y > heightSS(coord.x, 0)) alpha = max(alpha, 0.3);

    color = vec4(1.0, 0.0, 0.0, alpha);
}
