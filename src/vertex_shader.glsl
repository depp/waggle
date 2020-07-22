#version 330
in vec4 VertexPosition;
flat out vec4 Color;

uniform vec2 Scale;

layout (std140) uniform bitstring
{
    uint data[4];
} bs;

const float kLineWidth = 4.0;
const float kHeight = 40.0;
const float kWidth = 40.0;
const float kSpacing = 60.0;

uint get_bit(int index) {
    index = clamp(index, 0, 31);
    return (bs.data[gl_InstanceID + (index >> 5)] >> (index & 31)) & 1u;
}

void main() {
    // Every point in the line has a signal level to the left and one to the right.
    // We calculate "index" as the index of the bit for the signal to the right of this point.
    int index = (gl_VertexID + 2) >> 2;
    float level0 = float(get_bit(index - 1));
    float level1 = float(get_bit(index));
    // The signal level at this point in the line.
    float level = mix(level0, level1, (gl_VertexID & 2) == 0);
    vec2 pos = vec2(20.0, 20.0);
    // Move to (x, y) position of line.
    pos += vec2(float(index) * kWidth, float(level) * kHeight);
    // Add an offset to give the triangle strip width.
    pos += vec2((level0 - level1) * (float(gl_VertexID & 1) - 0.5), gl_VertexID & 1) * kLineWidth;
    // Move up for different signals in the set of signals.
    pos.y += float(gl_InstanceID) * kSpacing;
    // Color green for horizontal, red for vertical lines.
    Color = mix(vec4(0.0, 1.0, 0.0, 1.0), vec4(1.0, 0.0, 0.0, 1.0), (gl_VertexID & 2) == 0);
    gl_Position = vec4(pos * Scale - vec2(1.0), 0.0, 1.0);
}
