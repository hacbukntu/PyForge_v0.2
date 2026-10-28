// Estos módulos viajan dentro de pyforgec: no hay que descargar archivos .st adicionales.
importar "math";
importar "time";
importar "os";
importar "io";
importar "json";

// Cada módulo tiene su propio espacio de nombres; las funciones usan tipos explícitos.
mostrar(math.raiz(81.0));
mostrar(math.potencia(2.0, -3));
mostrar(time.ahora() > 0);
variable inicio: entero = time.monotono();
time.dormir(1);
mostrar(time.monotono() >= inicio);
mostrar(os.existe(os.directorio_actual()));
io.imprimir("Biblioteca estándar ST");

// Los compuestos JSON siguen siendo texto; se extraen y decodifican de forma explícita.
variable documento: cadena = "{\"nombre\":\"Ana\",\"datos\":[1,true,null,{\"activo\":false}]}";
mostrar(json.validar(documento));
mostrar(json.decodificar(json.obtener(documento, "nombre")));
mostrar(json.elemento(json.obtener(documento, "datos"), 3));
mostrar(json.codificar("Hola, mundo\n"));
mostrar(json.decodificar("\"\\u00f1 \\ud83d\\ude80\""));
mostrar(json.validar("[1,]"));
