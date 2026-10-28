// Núcleo 3D propio de ST. No abre una ventana ni simula que existe una GPU:
// modela escenas, matemáticas, cámara, mallas, materiales, luces, animación y colisión.
// El futuro backend gráfico consumirá estas mismas estructuras sin cambiar el juego.

clase Vector3 {
    variable x: flotante;
    variable componente_y: flotante;
    variable z: flotante;
}

implementar Vector3 {
    funcion sumar(otro: Vector3) -> Vector3 {
        retornar nuevo Vector3(yo.x + otro.x; yo.componente_y + otro.componente_y; yo.z + otro.z);
    }
    funcion restar(otro: Vector3) -> Vector3 {
        retornar nuevo Vector3(yo.x - otro.x; yo.componente_y - otro.componente_y; yo.z - otro.z);
    }
    funcion escalar(factor: flotante) -> Vector3 {
        retornar nuevo Vector3(yo.x * factor; yo.componente_y * factor; yo.z * factor);
    }
    funcion producto_punto(otro: Vector3) -> flotante {
        retornar yo.x * otro.x + yo.componente_y * otro.componente_y + yo.z * otro.z;
    }
    funcion magnitud() -> flotante {
        retornar aux_raiz(yo.x * yo.x + yo.componente_y * yo.componente_y + yo.z * yo.z);
    }
    funcion normalizar() -> Vector3 {
        variable magnitud_valor: flotante = yo.magnitud();
        si magnitud_valor == 0.0 { retornar nuevo Vector3(0.0; 0.0; 0.0); }
        retornar nuevo Vector3(yo.x / magnitud_valor; yo.componente_y / magnitud_valor; yo.z / magnitud_valor);
    }
    funcion distancia(otro: Vector3) -> flotante {
        retornar yo.restar(otro).magnitud();
    }
}

clase Cuaternion {
    variable x: flotante;
    variable componente_y: flotante;
    variable z: flotante;
    variable w: flotante;
}

implementar Cuaternion {
    funcion normalizar() -> Cuaternion {
        variable magnitud_valor: flotante = aux_raiz(yo.x * yo.x + yo.componente_y * yo.componente_y + yo.z * yo.z + yo.w * yo.w);
        si magnitud_valor == 0.0 { retornar nuevo Cuaternion(0.0; 0.0; 0.0; 1.0); }
        retornar nuevo Cuaternion(yo.x / magnitud_valor; yo.componente_y / magnitud_valor; yo.z / magnitud_valor; yo.w / magnitud_valor);
    }
}

clase Transformacion3D {
    variable posicion: Vector3;
    variable rotacion: Cuaternion;
    variable escala: Vector3;
}

// Matriz fila-mayor de 16 valores. El constructor público valida el tamaño.
clase Matriz4 {
    variable valores: lista[flotante];
}

implementar Matriz4 {
    funcion obtener(fila: entero; columna: entero) -> flotante {
        si fila < 0 o fila > 3 o columna < 0 o columna > 3 { lanzar "motor3d.Matriz4: índice inválido"; }
        retornar yo.valores[fila * 4 + columna];
    }
    funcion multiplicar(otra: Matriz4) -> Matriz4 {
        variable resultado: lista[flotante] = [];
        variable fila: entero = 0;
        variable columna: entero = 0;
        variable paso: entero = 0;
        variable suma: flotante = 0.0;
        mientras fila < 4 {
            columna = 0;
            mientras columna < 4 {
                paso = 0;
                suma = 0.0;
                mientras paso < 4 {
                    suma = suma + yo.obtener(fila; paso) * otra.obtener(paso; columna);
                    paso = paso + 1;
                }
                resultado.agregar(suma);
                columna = columna + 1;
            }
            fila = fila + 1;
        }
        retornar nuevo Matriz4(resultado);
    }
    funcion transformar_punto(punto: Vector3) -> Vector3 {
        variable tx: flotante = yo.obtener(0; 0) * punto.x + yo.obtener(0; 1) * punto.componente_y + yo.obtener(0; 2) * punto.z + yo.obtener(0; 3);
        variable ty: flotante = yo.obtener(1; 0) * punto.x + yo.obtener(1; 1) * punto.componente_y + yo.obtener(1; 2) * punto.z + yo.obtener(1; 3);
        variable tz: flotante = yo.obtener(2; 0) * punto.x + yo.obtener(2; 1) * punto.componente_y + yo.obtener(2; 2) * punto.z + yo.obtener(2; 3);
        variable tw: flotante = yo.obtener(3; 0) * punto.x + yo.obtener(3; 1) * punto.componente_y + yo.obtener(3; 2) * punto.z + yo.obtener(3; 3);
        si tw != 0.0 y tw != 1.0 { retornar nuevo Vector3(tx / tw; ty / tw; tz / tw); }
        retornar nuevo Vector3(tx; ty; tz);
    }
}

implementar Transformacion3D {
    funcion trasladar(delta: Vector3) -> nada {
        yo.posicion = yo.posicion.sumar(delta);
    }
    funcion cambiar_escala(nueva: Vector3) -> nada {
        yo.escala = nueva;
    }
}

clase Material3D {
    variable nombre: cadena;
    variable color: Vector3;
    variable metalico: flotante;
    variable rugosidad: flotante;
    variable shader: cadena;
    variable doble_cara: booleano;
}

clase Malla3D {
    variable nombre: cadena;
    variable vertices: lista[flotante];
    variable indices: lista[entero];
    variable material: Material3D;
    variable transformacion: Transformacion3D;
    variable visible: booleano;
}

implementar Malla3D {
    funcion cantidad_vertices() -> entero { retornar longitud(yo.vertices) / 3; }
    funcion cantidad_triangulos() -> entero { retornar longitud(yo.indices) / 3; }
    funcion validar() -> booleano {
        si longitud(yo.vertices) < 9 o longitud(yo.vertices) % 3 != 0 { retornar falso; }
        si longitud(yo.indices) < 3 o longitud(yo.indices) % 3 != 0 { retornar falso; }
        para i en rango(longitud(yo.indices)) {
            si yo.indices[i] < 0 o yo.indices[i] >= yo.cantidad_vertices() { retornar falso; }
        }
        retornar verdadero;
    }
}

clase Camara3D {
    variable posicion: Vector3;
    variable objetivo: Vector3;
    variable campo_vision: flotante;
    variable aspecto: flotante;
    variable plano_cercano: flotante;
    variable plano_lejano: flotante;
}

implementar Camara3D {
    funcion mirar(objetivo_nuevo: Vector3) -> nada { yo.objetivo = objetivo_nuevo; }
    funcion avanzar(distancia: flotante) -> nada {
        variable direccion: Vector3 = yo.objetivo.restar(yo.posicion).normalizar();
        variable movimiento: Vector3 = direccion.escalar(distancia);
        yo.posicion = yo.posicion.sumar(movimiento);
        yo.objetivo = yo.objetivo.sumar(movimiento);
    }
    funcion validar() -> booleano {
        retornar yo.campo_vision > 1.0 y yo.campo_vision < 179.0 y yo.aspecto > 0.0 y yo.plano_cercano > 0.0 y yo.plano_lejano > yo.plano_cercano;
    }
}

clase Luz3D {
    variable nombre: cadena;
    variable tipo_luz: cadena;
    variable posicion: Vector3;
    variable color: Vector3;
    variable intensidad: flotante;
    variable alcance: flotante;
}

implementar Luz3D {
    funcion validar() -> booleano {
        retornar yo.intensidad >= 0.0 y yo.alcance >= 0.0 y (yo.tipo_luz == "direccional" o yo.tipo_luz == "punto" o yo.tipo_luz == "foco");
    }
}

clase Caja3D {
    variable minimo: Vector3;
    variable maximo: Vector3;
}

// Integrador lineal estable para prototipos. Una física completa irá en backend opcional.
clase CuerpoRigido3D {
    variable transformacion: Transformacion3D;
    variable velocidad: Vector3;
    variable aceleracion: Vector3;
    variable masa: flotante;
    variable restitucion: flotante;
    variable estatico: booleano;
}

implementar CuerpoRigido3D {
    funcion impulso(fuerza: Vector3) -> nada {
        si no yo.estatico y yo.masa > 0.0 {
            yo.velocidad = yo.velocidad.sumar(fuerza.escalar(1.0 / yo.masa));
        }
    }
    funcion integrar(delta: flotante) -> nada {
        si delta < 0.0 o delta > 1.0 { lanzar "motor3d.CuerpoRigido3D: delta inválido"; }
        si no yo.estatico {
            yo.velocidad = yo.velocidad.sumar(yo.aceleracion.escalar(delta));
            yo.transformacion.trasladar(yo.velocidad.escalar(delta));
        }
    }
}

implementar Caja3D {
    funcion contiene(punto: Vector3) -> booleano {
        retornar punto.x >= yo.minimo.x y punto.x <= yo.maximo.x y punto.componente_y >= yo.minimo.componente_y y punto.componente_y <= yo.maximo.componente_y y punto.z >= yo.minimo.z y punto.z <= yo.maximo.z;
    }
    funcion intersecta(otra: Caja3D) -> booleano {
        retornar yo.minimo.x <= otra.maximo.x y yo.maximo.x >= otra.minimo.x y yo.minimo.componente_y <= otra.maximo.componente_y y yo.maximo.componente_y >= otra.minimo.componente_y y yo.minimo.z <= otra.maximo.z y yo.maximo.z >= otra.minimo.z;
    }
}

clase Animacion3D {
    variable nombre: cadena;
    variable inicio: Vector3;
    variable fin: Vector3;
    variable duracion: flotante;
    variable tiempo: flotante;
    variable repetir: booleano;
}

implementar Animacion3D {
    funcion reiniciar() -> nada { yo.tiempo = 0.0; }
    funcion actualizar(delta: flotante) -> Vector3 {
        si yo.duracion <= 0.0 { retornar yo.fin; }
        yo.tiempo = yo.tiempo + delta;
        si yo.repetir y yo.tiempo > yo.duracion { yo.tiempo = yo.tiempo - yo.duracion; }
        si yo.tiempo > yo.duracion { yo.tiempo = yo.duracion; }
        variable t: flotante = yo.tiempo / yo.duracion;
        retornar yo.inicio.escalar(1.0 - t).sumar(yo.fin.escalar(t));
    }
}

clase Escena3D {
    variable nombre: cadena;
    variable camara: Camara3D;
    variable malla_principal: Malla3D;
    variable luz_principal: Luz3D;
    variable color_ambiente: Vector3;
    variable activa: booleano;
}

implementar Escena3D {
    funcion validar() -> booleano {
        retornar yo.camara.validar() y yo.malla_principal.validar() y yo.luz_principal.validar();
    }
    funcion actualizar(delta: flotante) -> nada {
        si yo.activa y delta > 0.0 {
            // Punto de extensión determinista para física, scripts y animaciones.
            yo.malla_principal.transformacion.rotacion = yo.malla_principal.transformacion.rotacion.normalizar();
        }
    }
}

clase Motor3D {
    variable escena: Escena3D;
    variable ejecutando: booleano;
    variable cuadro: entero;
    variable tiempo: flotante;
    variable ancho: entero;
    variable alto: entero;
}

// Catálogo con presupuesto explícito. Conserva metadatos; el backend será dueño
// de buffers/texturas GPU y notificará aquí altas y liberaciones comprobables.
clase GestorRecursos3D {
    variable nombres: lista[cadena];
    variable tipos: lista[cadena];
    variable estados: lista[cadena];
    variable tamanos: lista[entero];
    variable presupuesto: entero;
    variable usado: entero;
}

implementar GestorRecursos3D {
    funcion registrar(nombre: cadena; tipo_recurso: cadena; bytes: entero) -> entero {
        si bytes < 0 { lanzar "motor3d.recursos: tamaño negativo"; }
        si yo.usado + bytes > yo.presupuesto { lanzar "motor3d.recursos: presupuesto agotado"; }
        para i en rango(longitud(yo.nombres)) {
            si yo.nombres[i] == nombre y yo.estados[i] != "liberado" {
                lanzar "motor3d.recursos: nombre duplicado";
            }
        }
        yo.nombres.agregar(nombre);
        yo.tipos.agregar(tipo_recurso);
        yo.estados.agregar("cargado");
        yo.tamanos.agregar(bytes);
        yo.usado = yo.usado + bytes;
        retornar longitud(yo.nombres) - 1;
    }
    funcion esta_cargado(id: entero) -> booleano {
        retornar id >= 0 y id < longitud(yo.estados) y yo.estados[id] == "cargado";
    }
    funcion liberar(id: entero) -> nada {
        si id < 0 o id >= longitud(yo.estados) { lanzar "motor3d.recursos: identificador inválido"; }
        si yo.estados[id] != "cargado" { lanzar "motor3d.recursos: recurso ya liberado"; }
        yo.usado = yo.usado - yo.tamanos[id];
        yo.estados[id] = "liberado";
    }
    funcion disponible() -> entero { retornar yo.presupuesto - yo.usado; }
}

implementar Motor3D {
    funcion iniciar() -> nada {
        si no yo.escena.validar() { lanzar "motor3d: la escena no es válida"; }
        si yo.ancho <= 0 o yo.alto <= 0 { lanzar "motor3d: resolución inválida"; }
        yo.ejecutando = verdadero;
        yo.cuadro = 0;
        yo.tiempo = 0.0;
    }
    funcion actualizar(delta: flotante) -> nada {
        si delta < 0.0 o delta > 1.0 { lanzar "motor3d: delta fuera del intervalo seguro"; }
        si yo.ejecutando {
            yo.escena.actualizar(delta);
            yo.tiempo = yo.tiempo + delta;
            yo.cuadro = yo.cuadro + 1;
        }
    }
    funcion detener() -> nada { yo.ejecutando = falso; }
}

funcion vector(x: flotante; componente_y: flotante; z: flotante) -> Vector3 {
    retornar nuevo Vector3(x; componente_y; z);
}

funcion cuaternion_identidad() -> Cuaternion {
    retornar nuevo Cuaternion(0.0; 0.0; 0.0; 1.0);
}

funcion transformacion_identidad() -> Transformacion3D {
    retornar nuevo Transformacion3D(vector(0.0; 0.0; 0.0); cuaternion_identidad(); vector(1.0; 1.0; 1.0));
}

funcion matriz_identidad() -> Matriz4 {
    retornar nuevo Matriz4([1.0; 0.0; 0.0; 0.0; 0.0; 1.0; 0.0; 0.0; 0.0; 0.0; 1.0; 0.0; 0.0; 0.0; 0.0; 1.0]);
}

funcion matriz_traslacion(posicion: Vector3) -> Matriz4 {
    retornar nuevo Matriz4([1.0; 0.0; 0.0; posicion.x; 0.0; 1.0; 0.0; posicion.componente_y; 0.0; 0.0; 1.0; posicion.z; 0.0; 0.0; 0.0; 1.0]);
}

funcion crear_cuerpo(posicion: Vector3; masa: flotante; estatico: booleano) -> CuerpoRigido3D {
    si masa <= 0.0 { lanzar "motor3d.crear_cuerpo: la masa debe ser positiva"; }
    variable transformacion: Transformacion3D = transformacion_identidad();
    transformacion.posicion = posicion;
    retornar nuevo CuerpoRigido3D(transformacion; vector(0.0; 0.0; 0.0); vector(0.0; -9.81; 0.0); masa; 0.2; estatico);
}

funcion material_estandar(nombre: cadena; rojo: flotante; verde: flotante; azul: flotante) -> Material3D {
    retornar nuevo Material3D(nombre; vector(rojo; verde; azul); 0.0; 0.5; "pbr"; falso);
}

funcion cubo(nombre: cadena; material: Material3D) -> Malla3D {
    variable vertices: lista[flotante] = [-1.0; -1.0; -1.0; 1.0; -1.0; -1.0; 1.0; 1.0; -1.0; -1.0; 1.0; -1.0; -1.0; -1.0; 1.0; 1.0; -1.0; 1.0; 1.0; 1.0; 1.0; -1.0; 1.0; 1.0];
    variable indices: lista[entero] = [0; 1; 2; 2; 3; 0; 4; 6; 5; 6; 4; 7; 0; 4; 5; 5; 1; 0; 3; 2; 6; 6; 7; 3; 1; 5; 6; 6; 2; 1; 0; 3; 7; 7; 4; 0];
    retornar nuevo Malla3D(nombre; vertices; indices; material; transformacion_identidad(); verdadero);
}

funcion crear_camara(ancho: entero; alto: entero) -> Camara3D {
    si ancho <= 0 o alto <= 0 { lanzar "motor3d.crear_camara: resolución inválida"; }
    retornar nuevo Camara3D(vector(0.0; 2.0; 6.0); vector(0.0; 0.0; 0.0); 60.0; flotante(ancho) / flotante(alto); 0.1; 1000.0);
}

funcion crear_escena(nombre: cadena; ancho: entero; alto: entero) -> Escena3D {
    variable material: Material3D = material_estandar("material_base"; 0.8; 0.7; 0.2);
    variable malla: Malla3D = cubo("cubo_principal"; material);
    variable camara: Camara3D = crear_camara(ancho; alto);
    variable luz: Luz3D = nuevo Luz3D("sol"; "direccional"; vector(3.0; 5.0; 4.0); vector(1.0; 1.0; 1.0); 2.0; 100.0);
    retornar nuevo Escena3D(nombre; camara; malla; luz; vector(0.05; 0.05; 0.08); verdadero);
}

funcion crear_motor(nombre: cadena; ancho: entero; alto: entero) -> Motor3D {
    retornar nuevo Motor3D(crear_escena(nombre; ancho; alto); falso; 0; 0.0; ancho; alto);
}

funcion crear_gestor_recursos(presupuesto_bytes: entero) -> GestorRecursos3D {
    si presupuesto_bytes <= 0 { lanzar "motor3d.recursos: presupuesto inválido"; }
    variable nombres: lista[cadena] = [];
    variable tipos: lista[cadena] = [];
    variable estados: lista[cadena] = [];
    variable tamanos: lista[entero] = [];
    retornar nuevo GestorRecursos3D(nombres; tipos; estados; tamanos; presupuesto_bytes; 0);
}
