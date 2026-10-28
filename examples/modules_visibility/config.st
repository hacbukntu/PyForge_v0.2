// API pública: otros módulos pueden leer version y llamar duplicar.
publica variable version: cadena = "1.0";

publica funcion duplicar(valor: entero) -> entero {
    retornar valor * 2;
}

// Detalles internos: no aparecen en importaciones con alias ni selectivas.
privada variable secreto: cadena = "solo-config";

privada funcion validar(valor: entero) -> booleano {
    retornar valor >= 0;
}
