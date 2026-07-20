import { useRef, useEffect, useState } from "react";
import {
  WebGLRenderer,
  Scene,
  PerspectiveCamera,
  BufferGeometry,
  BufferAttribute,
  ShaderMaterial,
  Points,
  Clock,
  AdditiveBlending,
} from "three";

const vertexShader = `
attribute float aOpacity;
attribute float aSize;
uniform float u_time;
varying float vOpacity;

void main() {
  float breathe = 1.0 + sin(u_time * 0.5) * 0.035;
  vec3 p = position * breathe;

  vec4 mvPosition = modelViewMatrix * vec4(p, 1.0);
  float dist = length(p.xy);
  vOpacity = aOpacity * (1.0 - dist * 0.045);
  gl_PointSize = aSize * (4.5 / -mvPosition.z);
  gl_Position = projectionMatrix * mvPosition;
}
`;

const fragmentShader = `
varying float vOpacity;
void main() {
  float d = length(gl_PointCoord - vec2(0.5));
  if (d > 0.5) discard;
  float alpha = smoothstep(0.5, 0.05, d) * vOpacity * 1.6;
  gl_FragColor = vec4(0.95, 0.94, 0.92, alpha);
}
`;

export function WebGLSpiral({ className }: { className?: string }) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [isVisible, setIsVisible] = useState(true);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const observer = new IntersectionObserver(([entry]) => setIsVisible(entry.isIntersecting), {
      threshold: 0,
    });
    observer.observe(canvas);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    if (!isVisible) return;
    const canvas = canvasRef.current;
    if (!canvas) return;

    const isMobile = window.innerWidth < 768;
    const PARTICLE_COUNT = isMobile ? 5000 : 12000;
    const ARMS = 8;
    const armSeparation = (Math.PI * 2) / ARMS;

    const positions = new Float32Array(PARTICLE_COUNT * 3);
    const opacities = new Float32Array(PARTICLE_COUNT);
    const sizes = new Float32Array(PARTICLE_COUNT);

    for (let i = 0; i < PARTICLE_COUNT; i++) {
      const armIndex = i % ARMS;
      const progress = Math.floor(i / ARMS) / (PARTICLE_COUNT / ARMS);
      const angle = armIndex * armSeparation + progress * Math.PI * 4;
      const radius = 0.25 + progress * 7.0;
      const jitter = (Math.random() - 0.5) * radius * 0.08;

      positions[i * 3] = Math.cos(angle) * (radius + jitter);
      positions[i * 3 + 1] = Math.sin(angle) * (radius + jitter);
      positions[i * 3 + 2] = (Math.random() - 0.5) * 0.35;
      opacities[i] = 0.6 + Math.random() * 0.4;
      sizes[i] = 0.9 + Math.random() * 1.8;
    }

    const width = canvas.clientWidth || window.innerWidth;
    const height = canvas.clientHeight || window.innerHeight;

    const scene = new Scene();
    const camera = new PerspectiveCamera(60, width / height, 0.1, 100);
    camera.position.set(0, 0, 3.6);

    const renderer = new WebGLRenderer({ canvas, alpha: true, antialias: false });
    renderer.setSize(width, height);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));

    const geometry = new BufferGeometry();
    geometry.setAttribute("position", new BufferAttribute(positions, 3));
    geometry.setAttribute("aOpacity", new BufferAttribute(opacities, 1));
    geometry.setAttribute("aSize", new BufferAttribute(sizes, 1));

    const material = new ShaderMaterial({
      vertexShader,
      fragmentShader,
      uniforms: { u_time: { value: 0 } },
      transparent: true,
      depthWrite: false,
      blending: AdditiveBlending,
    });

    const points = new Points(geometry, material);
    scene.add(points);

    const clock = new Clock();
    let animId: number;

    const animate = () => {
      animId = requestAnimationFrame(animate);
      material.uniforms.u_time.value = clock.getElapsedTime();
      points.rotation.z += 0.0008;
      renderer.render(scene, camera);
    };

    animate();

    const onResize = () => {
      const w = canvas.clientWidth || window.innerWidth;
      const h = canvas.clientHeight || window.innerHeight;
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
      renderer.setSize(w, h);
    };
    window.addEventListener("resize", onResize);

    const onContextLost = (event: Event) => {
      event.preventDefault();
      cancelAnimationFrame(animId);
    };
    canvas.addEventListener("webglcontextlost", onContextLost);

    return () => {
      cancelAnimationFrame(animId);
      window.removeEventListener("resize", onResize);
      canvas.removeEventListener("webglcontextlost", onContextLost);
      geometry.dispose();
      material.dispose();
      renderer.dispose();
    };
  }, [isVisible]);

  return (
    <canvas ref={canvasRef} className={className} style={{ display: "block" }} aria-hidden="true" />
  );
}
