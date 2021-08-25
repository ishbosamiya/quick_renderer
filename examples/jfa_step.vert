#version 330 core

uniform float step_size;

in vec3 in_pos;
in vec2 in_uv;

out float v_step_size;
out vec2 v_uv;

void main() {
	gl_Position = vec4(in_pos, 1.0);
	v_step_size = step_size;
	v_uv = in_uv;
}
