#version 330 core

#define JFA_NULL -1.0
#define JFA_NULL_VEC2 vec2(JFA_NULL, JFA_NULL)
#define JFA_NULL_VEC3 vec3(JFA_NULL_VEC2, 0.0)
#define JFA_NULL_VEC4 vec4(JFA_NULL_VEC3, 0.0)

uniform sampler2D u_image;

in vec2 v_uv;

out vec4 o_frag_color;

void main() {
	vec4 pixel = texture(u_image, v_uv);
	if ((pixel.r + pixel.g) > 0.0) {
		o_frag_color = vec4(v_uv, pixel.b, pixel.a);
	}
	else {
		o_frag_color = JFA_NULL_VEC4;
	}
}
