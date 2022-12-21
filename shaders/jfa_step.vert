#version 330 core

uniform float u_step_size;

in vec3 in_pos;
in vec2 in_uv;

out vec2 v_uv;

void main() {
	gl_Position = vec4(in_pos, 1.0);
	v_uv = in_uv;
}
