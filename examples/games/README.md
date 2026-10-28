# Juegos con ST

Estos ejemplos separan la lógica escrita en `.st` de los adaptadores nativos de
ventana, GPU y teclado. Cada archivo muestra una capacidad que existe en el proyecto;
no son plantillas de un motor AAA terminado.

## Archivos

- `snake.st`: serpiente de terminal con teclado inmediato.
- `pong.st`, `plataforma.st` y `breakout.st`: reglas 2D y colisiones.
- `motor_2d.st`, `motor_avanzado.st` y `01_motor_verificable.st`: capas de lógica
  reutilizable y comprobaciones deterministas.
- `motor_3d_nucleo.st`: escena 3D, cubo indexado, cámara, matrices, colisión,
  animación y cuerpo rígido escritos y ejecutados desde ST.
- [`JUEGOS_3D.md`](JUEGOS_3D.md): guía común de los cuatro juegos gráficos.
- [`portal_runner_visual`](portal_runner_visual/README.md): exploración y cinco portales.
- [`cristales_3d`](cristales_3d/README.md): recolección, drones y vidas.
- [`laberinto_3d`](laberinto_3d/README.md): navegación, muros y límite de turnos.
- [`guardianes_3d`](guardianes_3d/README.md): defensa, ataques y seis rondas.

El núcleo `motor3d` calcula el estado tridimensional, pero por sí solo no abre una
ventana. El crate opcional `crates/pyforge_engine3d` proporciona una demostración GPU,
un editor y un ejecutor de juegos con cubos sólidos sombreados y colores por material.
Estas rutas todavía no comparten un único grafo renderizado con PBR, sombras y
animación glTF.

## Reglas de diseño

- Escribe reglas y estado en ST siempre que el runtime actual lo permita.
- Mantén ventana, GPU y dispositivos en componentes nativos opcionales.
- Valida todos los datos que cruzan el proceso o la ABI.
- Documenta si un ejemplo usa terminal, viewport sólido o pipeline GPU.

## Qué usar como base

- clases y objetos para entidades
- listas para cuerpos, inventarios y colisiones
- funciones para lógica del juego
- condicionales y bucles para la actualización del estado

## Ejemplo de estructura

```st
clase Punto {
    variable x: entero;
    variable y: entero;
}

funcion mover_punto(p: Punto, dx: entero, dy: entero) -> nada {
    p.x = p.x + dx;
    p.y = p.y + dy;
}
```

Con esta base puedes construir reglas, tableros, avatares y escenas calculadas. Para
renderizado nativo consulta las secciones 35–37 de `docs/MANUAL_ST.md`.
