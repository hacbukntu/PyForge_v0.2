// Las colecciones comparten referencia al asignarlas o pasarlas a funciones.
variable numeros: lista = [10, 20, 30];
agregar(numeros, 40);
numeros[1] = 22;
mostrar(numeros[1]);
mostrar(numeros[-1]);
mostrar(longitud(numeros));
variable edades: diccionario = {"Ana": 20, "Luis": 30};
edades["Ana"] = 21;
edades["Eva"] = 18;
mostrar(edades["Ana"]);
mostrar(longitud(edades));
