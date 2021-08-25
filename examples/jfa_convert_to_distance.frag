#version 330 core

#define JFA_NULL -1.0
#define JFA_NULL_VEC2 vec2(JFA_NULL, JFA_NULL)
#define JFA_NULL_VEC3 vec3(JFA_NULL_VEC2, 0.0)
#define JFA_NULL_VEC4 vec4(JFA_NULL_VEC3, 0.0)
#define FLT_MAX 1.0e+20

uniform sampler2D image;

in vec2 v_uv;

out vec4 FragColor;

void main() {
	vec2 value = texture(image, v_uv).xy;

	float dist = FLT_MAX;
	if (value != JFA_NULL_VEC2) {
		dist = length(value - v_uv);
	}

	FragColor = vec4(vec3(dist), 1.0);
}
