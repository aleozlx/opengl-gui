#version 330 core
in vec2 position;
in vec2 vsTex;
out vec2 fsTex;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    fsTex = vsTex;
}
