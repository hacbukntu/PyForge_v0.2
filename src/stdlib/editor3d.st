// Puente declarativo entre ST y PyForge Editor 3D.
//
// Estas funciones no dibujan por sí solas. Publican un protocolo textual que el
// editor valida antes de crear la escena visual. Todas las transformaciones se
// expresan desde ST y pueden volver a exportarse desde los gizmos del editor.

// Inicia una escena. `cantidad` debe coincidir con las llamadas a entidad.
funcion iniciar(nombre: cadena; cantidad: entero) -> nada {
    mostrar("PYFORGE_EDITOR3D_V1");
    mostrar(nombre);
    mostrar(cantidad);
}

// Comienza una entidad. Una cadena vacía en malla o material significa que ese
// recurso opcional no está asignado. Las tres llamadas siguientes completan su
// transformación y respetan el máximo de seis parámetros del ABI de ST.
funcion entidad(nombre: cadena; malla: cadena; material: cadena) -> nada {
    mostrar(nombre);
    mostrar(malla);
    mostrar(material);
}

// Publica la posición XYZ de la última entidad declarada.
funcion posicion(posicion_x: flotante; posicion_y: flotante; posicion_z: flotante) -> nada {
    mostrar(posicion_x);
    mostrar(posicion_y);
    mostrar(posicion_z);
}

// Publica un cuaternión XYZW normalizado por el editor al usar los gizmos.
funcion rotacion(rotacion_x: flotante; rotacion_y: flotante; rotacion_z: flotante; rotacion_w: flotante) -> nada {
    mostrar(rotacion_x);
    mostrar(rotacion_y);
    mostrar(rotacion_z);
    mostrar(rotacion_w);
}

// Publica la escala XYZ; el editor rechaza componentes iguales a cero.
funcion escala(escala_x: flotante; escala_y: flotante; escala_z: flotante) -> nada {
    mostrar(escala_x);
    mostrar(escala_y);
    mostrar(escala_z);
}

// Cierra el protocolo. La marca permite detectar un programa interrumpido.
funcion terminar() -> nada {
    mostrar("FIN_PYFORGE_EDITOR3D_V1");
}
