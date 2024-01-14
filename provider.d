provider tracing {
    probe trace(char *);
    probe debug(char *);
    probe info(char *);
    probe warn(char *);
    probe error(char *);
    probe enter(char *);
    probe exit(char *);
    probe record(char *);
    probe event(char *);
};
