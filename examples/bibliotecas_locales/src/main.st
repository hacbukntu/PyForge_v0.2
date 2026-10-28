// @ busca una biblioteca del proyecto; el alias evita colisiones entre funciones.
importar "@estadistica" como estadistica;
variable ventas = [[10; 20]; [30; 40]];
mostrar(estadistica.sumar_filas(ventas)); // 100
mostrar(tipo(ventas)); // lista[lista[entero]]
