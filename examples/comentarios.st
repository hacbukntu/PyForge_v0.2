/*
   Los comentarios de bloque pueden ocupar varias líneas.
   El compilador ignora todo su contenido.
*/
variable nombre: cadena = "Ana";

"""
También puedes comentar con comillas triples.
mostrar("Esta instrucción no se ejecuta");
"""
mostrar("Hola, " + nombre); // Los comentarios anteriores siguen funcionando.
mostrar(10 /* comentario dentro de una expresión */ / 2);
# Las marcas dentro de una cadena se imprimen como texto.
mostrar("/* esto es texto */");
