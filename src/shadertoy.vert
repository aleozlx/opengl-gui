#version 330 core
in vec2 position;
in vec2 vsTex;
out vec2 fragCoord;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    // fragCoord = vsTex;
    fragCoord = position;
}
