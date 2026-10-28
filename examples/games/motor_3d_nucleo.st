// Escena 3D propia y verificable. Este ejemplo prueba el núcleo; aún no abre una ventana GPU.
importar "motor3d";

variable motor = motor3d.crear_motor("demostracion"; 1280; 720);
mostrar(motor.escena.nombre);
mostrar(motor.escena.malla_principal.cantidad_vertices());
mostrar(motor.escena.malla_principal.cantidad_triangulos());
mostrar(motor.escena.camara.validar());

motor.iniciar();
motor.actualizar(0.016);
motor.actualizar(0.016);
mostrar(motor.cuadro);
mostrar(motor.tiempo);

variable caja_a = nuevo motor3d.Caja3D(motor3d.vector(-1.0; -1.0; -1.0); motor3d.vector(1.0; 1.0; 1.0));
variable caja_b = nuevo motor3d.Caja3D(motor3d.vector(0.5; 0.5; 0.5); motor3d.vector(2.0; 2.0; 2.0));
mostrar(caja_a.intersecta(caja_b));

variable animacion = nuevo motor3d.Animacion3D("mover"; motor3d.vector(0.0; 0.0; 0.0); motor3d.vector(10.0; 0.0; 0.0); 2.0; 0.0; falso);
variable posicion = animacion.actualizar(1.0);
mostrar(posicion.x);

variable traslacion = motor3d.matriz_traslacion(motor3d.vector(2.0; 3.0; 4.0));
variable punto = traslacion.transformar_punto(motor3d.vector(1.0; 1.0; 1.0));
mostrar(punto.x);
mostrar(punto.componente_y);
mostrar(punto.z);

variable cuerpo = motor3d.crear_cuerpo(motor3d.vector(0.0; 10.0; 0.0); 2.0; falso);
cuerpo.impulso(motor3d.vector(2.0; 0.0; 0.0));
cuerpo.integrar(0.5);
mostrar(cuerpo.transformacion.posicion.x);
mostrar(cuerpo.transformacion.posicion.componente_y);

variable recursos = motor3d.crear_gestor_recursos(1048576);
variable id_malla = recursos.registrar("cubo"; "malla"; 1024);
mostrar(recursos.esta_cargado(id_malla));
mostrar(recursos.usado);
recursos.liberar(id_malla);
mostrar(recursos.disponible());
motor.detener();
