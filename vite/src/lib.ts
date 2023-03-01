export type IO = {
    in: string;
    out: string | null;
};

const builtin_examples: Array<{name: string, io: Array<IO>}> = [
    {
        name: "URLs",
        io: [
            {in: "http://www.example.com", out: "example"},
            {in: "https://www.apple.com/uk/mac", out: "apple"},
            {in: "https://www.google.com", out: null},
            {in: "www.mikail-khan.com", out: null},
        ]
    },
    {
        name: "Abbreviations",
        io: [
            {in: "First Last", out: "F.L."},
            {in: "Hi Aref", out: "H.A."},
            {in: "Bed Time", out: null},
            {in: "Another Name", out: null},
            {in: "Bhavesh Pareek", out: null},
            {in: "Saad Sharief", out: null},
        ]
    },
    {
        name: "Numbers",
        io: [
            {in: "I have 17 cookies", out: "17"},
            {in: "Give me at least 3 cookies", out: "3"},
            {in: "This number is 489", out: "489"},
            {in: "A string with the number 54234564 in the middle", out: null},
            {in: "36", out: null},
            {in: "Another 456432 string", out: ""},
        ]
    },
    {
        name: "Hello",
        io: [
            {in: "Hello", out: "Hello World"},
            {in: "Goodbye", out: "Goodbye World"},
            {in: "Hi", out: null},
            {in: "Patrick's", out: null},
            {in: "The", out: null},
        ]
    },
    {
        name: "Remove Between",
        io: [
            {in: "short /no/ line", out: "short  line"},
            {in: "aa/ /aa", out: "aaaa"},
            {in: "this breaks /down when longer/ outputs are given", out: null},
            {in: "/but of course/ it can run on longer test inputs", out: null},
        ]
    },
    {
        name: "Country Abbr Removal",
        io: [
            {in: "Mining US", out: "Mining"},
            {in: "Soybean Farming CAN", out: "Soybean Farming"},
            {in: "Mining", out: null},
            {in: "Oil Extraction US", out: null},
            {in: "Quarrying EU", out: null},
        ]
    },
];

export { builtin_examples }
