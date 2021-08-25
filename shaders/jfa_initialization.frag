#version 330 core

#define JFA_NULL -1.0
#define JFA_NULL_VEC2 vec2(JFA_NULL, JFA_NULL)
#define JFA_NULL_VEC3 vec3(JFA_NULL_VEC2, 0.0)
#define JFA_NULL_VEC4 vec4(JFA_NULL_VEC3, 0.0)

uniform sampler2D image;

in vec2 v_uv;

out vec4 FragColor;

// https://developer.mozilla.org/en-US/docs/Web/Accessibility/Understanding_Colors_and_Luminance
float rgb_to_luminance(vec3 rgb) {
	return dot(rgb, vec3(0.2126, 0.7152, 0.0722));
}

void main() {
	float lum = rgb_to_luminance(texture(image, v_uv).rgb);

	if (lum > 0.99) {
		FragColor = vec4(v_uv, 0.0, 0.0);
	}
	else {
		FragColor = JFA_NULL_VEC4;
	}
}
