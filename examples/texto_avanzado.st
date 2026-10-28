// El índice cuenta caracteres Unicode; negativo cuenta desde el final.
variable nombre: cadena = "Ana 😀";
mostrar(nombre[-1]);
mostrar(buscar(nombre, "😀"));
mostrar(reemplazar(nombre, "Ana", "Luis"));

// + y += convierten escalares si hay una cadena, sin cambiar la aritmética numérica.
variable edad: entero = 20;
mostrar("edad: " + edad);
mostrar(interpolar("Nombre: {}, edad: {}, precio: {}", nombre, edad, 12.5));

// Las claves pueden calcularse: se compara su contenido, no la dirección en memoria.
variable clave = "ed" + "ad";
variable datos = {clave: edad};
datos["edad"] = 21;
mostrar(datos[clave]);
