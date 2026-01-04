#version 330 core

out vec4 color;
in vec2 fragCoord;

uniform vec2 size; // The size on screen
uniform vec4 bgColor;
uniform vec4 traceColor;
uniform sampler2D text;
uniform float maxTraces;
uniform vec4 limits; // min_x, min_y, max_x, max_y

const float lineWidth = 4.0;
const float lineHalfWidth = lineWidth / 2.0;
const float loopBound = ceil(lineHalfWidth) + 1.0;

// Params:
// - x       : x normalized UV coordinates [0.0-1.0]
// - channel : the channel or trace (0-indexed)
float height(float x, int channel) {
    return (texture(text, vec2(x, channel / maxTraces)).r);
}

float distanceToLine(vec2 a, vec2 b, vec2 p) {
    float squaredLineLength = dot(b - a, b - a);
    float t = clamp(dot(p - a, b - a) / squaredLineLength, 0., 1.);
    return distance(p, a + t * (b - a));
}

void main() {
    // float dist = lineHalfWidth + 1.0;
    // vec2 previousPoint = vec2(fragCoord.x - lineHalfWidth, height(fragCoord.x - lineHalfWidth, 0));
    // for (float i = -loopBound + 1.0; i <= loopBound; i += 1) {
    //     vec2 currentPoint = vec2(fragCoord.x + i, height(fragCoord.x + i, 0));
    //     dist = min(dist, distanceToLine(previousPoint, currentPoint, fragCoord));
    //     previousPoint = currentPoint;
    // }

    float dist = abs(height(fragCoord.x, 0) - fragCoord.y);

    if (dist < 0.05) {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    } else {
        color = vec4(0.0, 0.0, 0.0, 1.0);
    }
    // color = vec4(height(fragCoord.x, 0), 0.0, 0.0, 1.0);
    //float magnitude = clamp(lineHalfWidth + 0.5 - dist, 0.0, 1.0);
}
