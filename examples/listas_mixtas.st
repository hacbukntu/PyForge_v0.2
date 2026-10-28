// Lista ordenada y mutable con elementos de tipos diferentes.
// El compilador infiere lista[mixto] porque el literal no es homogéneo.
variable perfil = ["Ada"; 36; 1.65; verdadero; nulo];

mostrar(tipo(perfil));    // lista[mixto]
mostrar(perfil[0]);       // Ada
mostrar(tipo(perfil[1])); // entero

// La posición conserva el orden, pero puede recibir después otro tipo.
perfil[1] = "36 años";
perfil.agregar(2026);
agregar(perfil; falso);

mostrar(perfil[1]);
mostrar(perfil[-2]);
mostrar(perfil[-1]);

// Una lista vacía necesita la anotación para aceptar tipos diferentes.
variable eventos: lista[mixto] = [];
eventos.agregar("inicio");
eventos.agregar(10);
eventos.agregar(2.5);

// tipo() consulta la etiqueta real de cada elemento recuperado.
para indice en rango(longitud(eventos)) {
    mostrar(tipo(eventos[indice]));
    mostrar(eventos[indice]);
}

// Las conversiones son explícitas y verificadas en ejecución.
variable entrada_mixta = ["42"; 8];
mostrar(entero(entrada_mixta[0]) + entero(entrada_mixta[1])); // 50

